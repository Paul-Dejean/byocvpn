use async_trait::async_trait;
use byocvpn_core::{
    daemon_client::{DaemonClient, DaemonCommand},
    error::Result,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind},
    net::UnixStream,
    time::{Duration, sleep},
};

use crate::constants;
pub struct UnixDaemonClient;

#[async_trait]
impl DaemonClient for UnixDaemonClient {
    async fn send_command(&self, cmd: DaemonCommand) -> Result<String> {
        let socket_path = constants::socket_path().to_string_lossy().to_string();
        wait_for_socket(&socket_path, 50).await?;
        let mut stream = UnixStream::connect(&socket_path).await?;
        println!("Connected to daemon at {}", &socket_path);
        // Serializing DaemonCommand should never fail as it's a simple enum
        let msg = serde_json::to_string(&cmd).expect("Failed to serialize DaemonCommand") + "\n";
        stream.write_all(msg.as_bytes()).await?;

        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    }
    async fn is_daemon_running(&self) -> bool {
        let socket_path = constants::socket_path().to_string_lossy().to_string();
        match UnixStream::connect(&socket_path).await {
            Ok(_) => {
                println!("Successfully connected to daemon at {}", &socket_path);
                true
            } // Connection succeeded — daemon is alive
            Err(e) => {
                // Connection failed — probably not running
                println!("Daemon connection error: {:?}", e);
                false
            }
        }
    }
}

async fn wait_for_socket(path: &str, max_retries: u32) -> std::io::Result<()> {
    for _ in 0..max_retries {
        match UnixStream::connect(path).await {
            Ok(_) => return Ok(()), // Daemon is up and accepting
            Err(e)
                if e.kind() == ErrorKind::ConnectionRefused || e.kind() == ErrorKind::NotFound =>
            {
                sleep(Duration::from_millis(100)).await;
            }
            Err(e) => return Err(e), // Any other error (e.g., permission denied)
        }
    }

    Err(Error::new(
        ErrorKind::TimedOut,
        "Timed out waiting for daemon socket",
    ))
}
