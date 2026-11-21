use std::{
    fs::remove_file as remove_file_blocking,
    io::ErrorKind,
    mem::{replace, size_of, zeroed},
    path::{Path, PathBuf},
    ptr::null,
};

use log::warn;
use tokio::{
    fs::{remove_file, try_exists},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
    time::{Duration, sleep},
};
use windows_sys::Win32::Security::{
    ACL, InitializeSecurityDescriptor, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
    SECURITY_DESCRIPTOR, SetSecurityDescriptorDacl,
};

use crate::error::DaemonError;

const ERROR_PIPE_BUSY: i32 = 231;
const SECURITY_DESCRIPTOR_REVISION: u32 = 1;
const PIPE_BUSY_RETRY_DELAY: Duration = Duration::from_millis(50);

#[derive(Debug)]
enum NamedPipeStream {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

#[derive(Debug)]
pub struct IpcSocket {
    pipe_server: NamedPipeServer,
    path: PathBuf,
}

#[derive(Debug)]
pub struct IpcStream {
    stream: NamedPipeStream,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self, DaemonError> {
        if try_exists(&path).await.unwrap_or(false)
            && let Err(error) = remove_file(&path).await
        {
            warn!(
                "failed to remove stale pipe path {}: {error}",
                path.display()
            );
        }

        let server = create_pipe_server_with_null_dacl(&path, true)?;

        Ok(Self {
            pipe_server: server,
            path,
        })
    }

    pub async fn accept(&mut self) -> Result<IpcStream, DaemonError> {
        self.pipe_server.connect().await?;
        let next_server = create_pipe_server_with_null_dacl(&self.path, false)?;
        let connected = replace(&mut self.pipe_server, next_server);
        Ok(IpcStream {
            stream: NamedPipeStream::Server(connected),
        })
    }

    pub const fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl IpcStream {
    pub async fn connect(path: &Path) -> Result<Self, DaemonError> {
        loop {
            match ClientOptions::new().open(path) {
                Ok(client) => {
                    return Ok(Self {
                        stream: NamedPipeStream::Client(client),
                    });
                }
                Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    sleep(PIPE_BUSY_RETRY_DELAY).await;
                }
                Err(error) => return Err(DaemonError::Socket(error)),
            }
        }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<(), DaemonError> {
        match &mut self.stream {
            NamedPipeStream::Server(server) => {
                server.write_all(message.as_bytes()).await?;
                server.write_all(b"\n").await?;
            }
            NamedPipeStream::Client(client) => {
                client.write_all(message.as_bytes()).await?;
                client.write_all(b"\n").await?;
            }
        }
        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<String>, DaemonError> {
        let mut line = String::new();
        let bytes_read = match &mut self.stream {
            NamedPipeStream::Server(server) => BufReader::new(server).read_line(&mut line).await,
            NamedPipeStream::Client(client) => BufReader::new(client).read_line(&mut line).await,
        }?;

        if bytes_read == 0 {
            return Ok(None);
        }
        if line.ends_with('\n') {
            line.pop();
        }
        Ok(Some(line))
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), DaemonError> {
        match &mut self.stream {
            NamedPipeStream::Server(server) => server.write_all(data).await,
            NamedPipeStream::Client(client) => client.write_all(data).await,
        }?;
        Ok(())
    }
}

impl Drop for IpcSocket {
    fn drop(&mut self) {
        if let Err(error) = remove_file_blocking(&self.path)
            && error.kind() != ErrorKind::NotFound
        {
            warn!(
                "failed to remove pipe path {}: {error}",
                self.path.display()
            );
        }
    }
}

fn create_pipe_server_with_null_dacl(
    path: &Path,
    first_instance: bool,
) -> Result<NamedPipeServer, DaemonError> {
    let mut security_descriptor: SECURITY_DESCRIPTOR = unsafe { zeroed() };
    let security_descriptor_ptr: PSECURITY_DESCRIPTOR = (&raw mut security_descriptor).cast();
    unsafe { InitializeSecurityDescriptor(security_descriptor_ptr, SECURITY_DESCRIPTOR_REVISION) };
    unsafe { SetSecurityDescriptorDacl(security_descriptor_ptr, 1_i32, null::<ACL>(), 0_i32) };

    let mut security_attributes = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>()).unwrap_or(u32::MAX),
        lpSecurityDescriptor: security_descriptor_ptr,
        bInheritHandle: 0,
    };

    let server = unsafe {
        ServerOptions::new()
            .first_pipe_instance(first_instance)
            .create_with_security_attributes_raw(path, (&raw mut security_attributes).cast())
    }?;
    Ok(server)
}
