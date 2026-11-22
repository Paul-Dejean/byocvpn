use std::{
    fs::remove_file as remove_file_blocking,
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use tokio::{
    fs::{metadata, remove_file, set_permissions},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tracing::{debug, warn};

use crate::error::DaemonError;

#[derive(Debug)]
pub struct IpcSocket {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcSocket {
    pub async fn bind(path: PathBuf) -> Result<Self, DaemonError> {
        remove_stale_socket_file(&path).await;

        let listener = UnixListener::bind(&path)?;

        let mut permissions = metadata(&path).await?.permissions();
        permissions.set_mode(0o777);
        set_permissions(&path, permissions).await?;

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
    match remove_file(path).await {
        Ok(()) => debug!("removed stale socket file at {}", path.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => warn!(%error, "failed to remove stale socket file at {}",
            path.display()
        ),
    }
}

impl Drop for IpcSocket {
    fn drop(&mut self) {
        if let Err(error) = remove_file_blocking(&self.path)
            && error.kind() != ErrorKind::NotFound
        {
            warn!(%error, "failed to remove socket file at {}",
                self.path.display()
            );
        }
    }
}

#[derive(Debug)]
pub struct IpcStream {
    stream: UnixStream,
}

impl IpcStream {
    pub async fn connect(path: &Path) -> Result<Self, DaemonError> {
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
