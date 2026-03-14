use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use crate::error::{DaemonError, Result};

pub struct IpcSocket {
    #[cfg(unix)]
    listener: UnixListener,
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
            unimplemented!("Windows named pipes not yet implemented")
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
            unimplemented!("Windows named pipes not yet implemented")
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
            unimplemented!("Windows named pipes not yet implemented")
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
            unimplemented!("Windows named pipes not yet implemented")
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
            unimplemented!("Windows named pipes not yet implemented")
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
            unimplemented!("Windows named pipes not yet implemented")
        }
    }
}
