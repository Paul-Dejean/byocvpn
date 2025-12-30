use async_trait::async_trait;
use byocvpn_core::{
    daemon_client::{DaemonClient, DaemonCommand},
    error::{DaemonError, Error, Result},
    ipc::IpcStream,
};
use tokio::time::{Duration, sleep};

use crate::constants;

pub struct UnixDaemonClient;

#[async_trait]
impl DaemonClient for UnixDaemonClient {
    async fn send_command(&self, cmd: DaemonCommand) -> Result<String> {
        if !self.is_daemon_running().await {
            return Err(DaemonError::NotRunning.into());
        }
        let socket_path = constants::socket_path();
        wait_for_socket(&socket_path, 50).await?;

        let mut stream = IpcStream::connect(&socket_path).await?;
        println!("Connected to daemon at {}", socket_path.to_string_lossy());

        // Serializing DaemonCommand should never fail as it's a simple enum
        let msg = serde_json::to_string(&cmd).map_err(|error| Error::Json(error))?;
        stream.send_message(&msg).await?;

        let response = stream.read_message().await?.ok_or_else(|| {
            Error::InputOutput(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Daemon closed connection without response",
            ))
        })?;
        Ok(response)
    }

    async fn is_daemon_running(&self) -> bool {
        let socket_path = constants::socket_path();

        // Try to send a health check command
        match IpcStream::connect(&socket_path).await {
            Ok(mut stream) => {
                // Send health check command
                let health_cmd = DaemonCommand::HealthCheck;
                let msg = match serde_json::to_string(&health_cmd) {
                    Ok(m) => m,
                    Err(_) => return false,
                };

                if stream.send_message(&msg).await.is_err() {
                    return false;
                }

                // Try to read response
                match stream.read_message().await {
                    Ok(Some(_)) => {
                        println!("Daemon is healthy at {}", socket_path.to_string_lossy());
                        true
                    }
                    _ => {
                        println!("Daemon did not respond to health check");
                        false
                    }
                }
            }
            Err(e) => {
                println!("Daemon connection error: {:?}", e);
                false
            }
        }
    }
}

async fn wait_for_socket(path: &std::path::PathBuf, max_retries: u32) -> Result<()> {
    for _ in 0..max_retries {
        match IpcStream::connect(path).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    Err(Error::InputOutput(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Timed out waiting for daemon socket",
    )))
}
