// filepath: /Users/paul/projects/on-demand-vpn/crates/core/src/ipc.rs
use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use crate::error::Result;

/// Platform-agnostic IPC socket wrapper
pub struct IpcSocket {
    #[cfg(unix)]
    listener: UnixListener,
    path: PathBuf,
}

impl IpcSocket {
    /// Create a new IPC socket at the given path
    pub async fn bind(path: PathBuf) -> Result<Self> {
        // Remove old socket if it exists
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            let _ = tokio::fs::remove_file(&path).await;
        }

        #[cfg(unix)]
        {
            let listener = UnixListener::bind(&path)?;

            // Set socket permissions
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
            // TODO: Implement Windows named pipes
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Accept a new connection
    pub async fn accept(&self) -> Result<IpcStream> {
        #[cfg(unix)]
        {
            let (stream, _) = self.listener.accept().await?;
            Ok(IpcStream { stream })
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Get the socket path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for IpcSocket {
    fn drop(&mut self) {
        // Clean up socket file on drop
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Platform-agnostic IPC stream wrapper
pub struct IpcStream {
    #[cfg(unix)]
    stream: UnixStream,
}

impl IpcStream {
    /// Connect to an IPC socket
    pub async fn connect(path: &PathBuf) -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = UnixStream::connect(path).await?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Send a message (writes data followed by newline)
    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        #[cfg(unix)]
        {
            self.stream.write_all(message.as_bytes()).await?;
            self.stream.write_all(b"\n").await?;
            Ok(())
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Read a message (reads until newline)
    pub async fn read_message(&mut self) -> Result<Option<String>> {
        #[cfg(unix)]
        {
            let mut reader = BufReader::new(&mut self.stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await? {
                0 => Ok(None), // EOF
                _ => {
                    // Remove trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
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

    /// Write raw data to the stream
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        #[cfg(unix)]
        {
            self.stream.write_all(data).await?;
            Ok(())
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Split the stream into read and write halves
    pub fn into_split(self) -> (IpcReadHalf, IpcWriteHalf) {
        #[cfg(unix)]
        {
            let (read, write) = self.stream.into_split();
            (IpcReadHalf { read }, IpcWriteHalf { write })
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }
}

/// Read half of an IPC stream
pub struct IpcReadHalf {
    #[cfg(unix)]
    read: tokio::net::unix::OwnedReadHalf,
}

impl IpcReadHalf {
    /// Create a buffered reader
    pub fn into_buf_reader(self) -> IpcBufReader {
        #[cfg(unix)]
        {
            IpcBufReader {
                reader: BufReader::new(self.read).lines(),
            }
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }
}

/// Write half of an IPC stream
pub struct IpcWriteHalf {
    #[cfg(unix)]
    write: tokio::net::unix::OwnedWriteHalf,
}

impl IpcWriteHalf {
    /// Send a message (writes data followed by newline)
    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        #[cfg(unix)]
        {
            self.write.write_all(message.as_bytes()).await?;
            self.write.write_all(b"\n").await?;
            Ok(())
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Write raw data to the stream
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        #[cfg(unix)]
        {
            self.write.write_all(data).await?;
            Ok(())
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }
}

/// Buffered reader for IPC streams
pub struct IpcBufReader {
    #[cfg(unix)]
    reader: tokio::io::Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
}

impl IpcBufReader {
    /// Read the next message (reads until newline)
    pub async fn read_message(&mut self) -> Result<Option<String>> {
        #[cfg(unix)]
        {
            let line = self.reader.next_line().await?;
            Ok(line)
        }

        #[cfg(windows)]
        {
            unimplemented!("Windows named pipes not yet implemented")
        }
    }

    /// Legacy method name for compatibility
    pub async fn next_line(&mut self) -> Result<Option<String>> {
        self.read_message().await
    }
}
