use std::{
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use log::{debug, warn};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::error::DaemonError;

pub struct IpcSocket {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self, DaemonError> {
        remove_stale_socket_file(&path).await;

        let listener = UnixListener::bind(&path)?;

        let mut permissions = tokio::fs::metadata(&path).await?.permissions();
        permissions.set_mode(0o777);
        tokio::fs::set_permissions(&path, permissions).await?;

        Ok(Self { listener, path })
    }

    pub async fn accept(&mut self) -> Result<IpcStream, DaemonError> {
        let (stream, _) = self.listener.accept().await?;
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
    pub async fn connect(path: &PathBuf) -> Result<Self, DaemonError> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub async fn send_message(&mut self, message: &str) -> Result<(), DaemonError> {
        self.stream.write_all(message.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<String>, DaemonError> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            Ok(None)
        } else {
            if line.ends_with('\n') {
                line.pop();
            }
            Ok(Some(line))
        }
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), DaemonError> {
        self.stream.write_all(data).await?;
        Ok(())
    }
}
