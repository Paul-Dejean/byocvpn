use std::{
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use log::{debug, warn};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::error::{DaemonError, Result};

pub struct IpcSocket {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self> {
        remove_stale_socket_file(&path).await;

        let listener = UnixListener::bind(&path).map_err(|error| DaemonError::SocketError {
            reason: format!("failed to bind socket at {}: {}", path.display(), error),
        })?;

        let mut permissions = tokio::fs::metadata(&path)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to stat socket at {}: {}", path.display(), error),
            })?
            .permissions();
        permissions.set_mode(0o777);
        tokio::fs::set_permissions(&path, permissions)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!(
                    "failed to set permissions on socket at {}: {}",
                    path.display(),
                    error
                ),
            })?;

        Ok(Self { listener, path })
    }

    pub async fn accept(&mut self) -> Result<IpcStream> {
        let (stream, _) =
            self.listener
                .accept()
                .await
                .map_err(|error| DaemonError::SocketError {
                    reason: format!("failed to accept connection: {error}"),
                })?;
        Ok(IpcStream { stream })
    }

    pub const fn path(&self) -> &PathBuf {
        &self.path
    }
}

async fn remove_stale_socket_file(path: &Path) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => debug!("removed stale socket file at {}", path.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => warn!(
            "failed to remove stale socket file at {}: {error}",
            path.display()
        ),
    }
}

impl Drop for IpcSocket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub struct IpcStream {
    stream: UnixStream,
}

impl IpcStream {
    pub async fn connect(path: &PathBuf) -> Result<Self> {
        let stream =
            UnixStream::connect(path)
                .await
                .map_err(|error| DaemonError::ConnectionFailed {
                    reason: error.to_string(),
                })?;
        Ok(Self { stream })
    }

    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        self.stream
            .write_all(message.as_bytes())
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to send message: {error}"),
            })?;
        self.stream
            .write_all(b"\n")
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to send message terminator: {error}"),
            })?;
        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<String>> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to read message: {error}"),
            })?
            == 0
        {
            Ok(None)
        } else {
            if line.ends_with('\n') {
                line.pop();
            }
            Ok(Some(line))
        }
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.stream
            .write_all(data)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to write data: {error}"),
            })?;
        Ok(())
    }
}
