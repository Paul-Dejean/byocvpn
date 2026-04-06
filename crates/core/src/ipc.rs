use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions};

use crate::error::{DaemonError, Result};

#[cfg(windows)]
enum NamedPipeStream {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

pub struct IpcSocket {
    #[cfg(unix)]
    listener: UnixListener,
    #[cfg(windows)]
    pipe_server: tokio::sync::Mutex<NamedPipeServer>,
    path: PathBuf,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self> {
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            let _ = tokio::fs::remove_file(&path).await;
        }

        #[cfg(unix)]
        {
            let listener = UnixListener::bind(&path).map_err(|error| DaemonError::SocketError {
                reason: format!("failed to bind socket at {}: {}", path.display(), error),
            })?;

            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o777);
                let _ = std::fs::set_permissions(&path, perms);
            }

            Ok(Self { listener, path })
        }

        #[cfg(windows)]
        {
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .max_instances(10)
                .create(&path)
                .map_err(|error| DaemonError::SocketError {
                    reason: format!(
                        "failed to create named pipe at {}: {}",
                        path.display(),
                        error
                    ),
                })?;

            Ok(Self { pipe_server: tokio::sync::Mutex::new(server), path })
        }
    }

    pub async fn accept(&self) -> Result<IpcStream> {
        #[cfg(unix)]
        {
            let (stream, _) = self.listener.accept().await.map_err(|error| {
                DaemonError::SocketError {
                    reason: format!("failed to accept connection: {}", error),
                }
            })?;
            Ok(IpcStream { stream })
        }

        #[cfg(windows)]
        {
            let mut server = self.pipe_server.lock().await;
            server.connect().await.map_err(|error| DaemonError::SocketError {
                reason: format!("failed to accept named pipe connection: {}", error),
            })?;
            let next_server =
                ServerOptions::new().max_instances(10).create(&self.path).map_err(|error| {
                    DaemonError::SocketError {
                        reason: format!(
                            "failed to create next named pipe instance: {}",
                            error
                        ),
                    }
                })?;
            let connected = std::mem::replace(&mut *server, next_server);
            Ok(IpcStream { stream: NamedPipeStream::Server(connected) })
        }
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

pub struct IpcStream {
    #[cfg(unix)]
    stream: UnixStream,
    #[cfg(windows)]
    stream: NamedPipeStream,
}

impl IpcStream {
    pub async fn connect(path: &PathBuf) -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = UnixStream::connect(path).await.map_err(|error| {
                DaemonError::ConnectionFailed {
                    reason: error.to_string(),
                }
            })?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            loop {
                match ClientOptions::new().open(path) {
                    Ok(client) => {
                        return Ok(Self { stream: NamedPipeStream::Client(client) });
                    }
                    Err(error) if error.raw_os_error() == Some(231) => {
                        // ERROR_PIPE_BUSY — all instances busy, wait and retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    Err(error) => {
                        return Err(DaemonError::ConnectionFailed {
                            reason: error.to_string(),
                        }
                        .into());
                    }
                }
            }
        }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        #[cfg(unix)]
        {
            self.stream
                .write_all(message.as_bytes())
                .await
                .map_err(|error| DaemonError::SocketError {
                    reason: format!("failed to send message: {}", error),
                })?;
            self.stream
                .write_all(b"\n")
                .await
                .map_err(|error| DaemonError::SocketError {
                    reason: format!("failed to send message terminator: {}", error),
                })?;
            Ok(())
        }

        #[cfg(windows)]
        {
            match &mut self.stream {
                NamedPipeStream::Server(server) => {
                    server.write_all(message.as_bytes()).await.map_err(|error| {
                        DaemonError::SocketError {
                            reason: format!("failed to send message: {}", error),
                        }
                    })?;
                    server.write_all(b"\n").await.map_err(|error| DaemonError::SocketError {
                        reason: format!("failed to send message terminator: {}", error),
                    })?;
                }
                NamedPipeStream::Client(client) => {
                    client.write_all(message.as_bytes()).await.map_err(|error| {
                        DaemonError::SocketError {
                            reason: format!("failed to send message: {}", error),
                        }
                    })?;
                    client.write_all(b"\n").await.map_err(|error| DaemonError::SocketError {
                        reason: format!("failed to send message terminator: {}", error),
                    })?;
                }
            }
            Ok(())
        }
    }

    pub async fn read_message(&mut self) -> Result<Option<String>> {
        #[cfg(unix)]
        {
            let mut reader = BufReader::new(&mut self.stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await.map_err(|error| DaemonError::SocketError {
                reason: format!("failed to read message: {}", error),
            })? {
                0 => Ok(None),
                _ => {
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    Ok(Some(line))
                }
            }
        }

        #[cfg(windows)]
        {
            let mut line = String::new();
            let bytes_read = match &mut self.stream {
                NamedPipeStream::Server(server) => {
                    BufReader::new(server).read_line(&mut line).await
                }
                NamedPipeStream::Client(client) => {
                    BufReader::new(client).read_line(&mut line).await
                }
            }
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to read message: {}", error),
            })?;

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
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        #[cfg(unix)]
        {
            self.stream.write_all(data).await.map_err(|error| DaemonError::SocketError {
                reason: format!("failed to write data: {}", error),
            })?;
            Ok(())
        }

        #[cfg(windows)]
        {
            match &mut self.stream {
                NamedPipeStream::Server(server) => server.write_all(data).await,
                NamedPipeStream::Client(client) => client.write_all(data).await,
            }
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to write data: {}", error),
            })?;
            Ok(())
        }
    }
}
