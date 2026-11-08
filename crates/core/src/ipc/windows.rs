use std::path::PathBuf;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
};
use windows_sys::Win32::Security::{
    ACL, InitializeSecurityDescriptor, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
    SECURITY_DESCRIPTOR, SetSecurityDescriptorDacl,
};

use crate::error::DaemonError;

const ERROR_PIPE_BUSY: i32 = 231;

enum NamedPipeStream {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

pub struct IpcSocket {
    pipe_server: NamedPipeServer,
    path: PathBuf,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self, DaemonError> {
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            let _ = tokio::fs::remove_file(&path).await;
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
        let connected = std::mem::replace(&mut self.pipe_server, next_server);
        Ok(IpcStream {
            stream: NamedPipeStream::Server(connected),
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for IpcSocket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn create_pipe_server_with_null_dacl(
    path: &PathBuf,
    first_instance: bool,
) -> Result<NamedPipeServer, DaemonError> {
    unsafe {
        let mut security_descriptor: SECURITY_DESCRIPTOR = std::mem::zeroed();
        let security_descriptor_ptr: PSECURITY_DESCRIPTOR =
            std::ptr::addr_of_mut!(security_descriptor).cast();
        InitializeSecurityDescriptor(security_descriptor_ptr, 1);
        SetSecurityDescriptorDacl(security_descriptor_ptr, 1i32, std::ptr::null::<ACL>(), 0i32);

        let mut security_attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: security_descriptor_ptr,
            bInheritHandle: 0,
        };

        Ok(ServerOptions::new()
            .first_pipe_instance(first_instance)
            .create_with_security_attributes_raw(
                path,
                std::ptr::addr_of_mut!(security_attributes).cast(),
            )?)
    }
}

pub struct IpcStream {
    stream: NamedPipeStream,
}

impl IpcStream {
    pub async fn connect(path: &PathBuf) -> Result<Self, DaemonError> {
        loop {
            match ClientOptions::new().open(path) {
                Ok(client) => {
                    return Ok(Self {
                        stream: NamedPipeStream::Client(client),
                    });
                }
                Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
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

        match bytes_read {
            0 => Ok(None),
            _ => {
                if line.ends_with('\n') {
                    line.pop();
                }
                Ok(Some(line))
            }
        }
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), DaemonError> {
        match &mut self.stream {
            NamedPipeStream::Server(server) => server.write_all(data).await,
            NamedPipeStream::Client(client) => client.write_all(data).await,
        }?;
        Ok(())
    }
}
