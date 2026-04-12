use async_trait::async_trait;
use byocvpn_core::{
    daemon_client::{DaemonClient, DaemonCommand},
    error::{DaemonError, Result},
    ipc::IpcStream,
};
use tokio::time::{Duration, sleep};

use crate::constants;
use log::*;

pub struct UnixDaemonClient;

#[async_trait]
impl DaemonClient for UnixDaemonClient {
    async fn send_command(&self, command: DaemonCommand) -> Result<String> {
        if !self.is_daemon_running().await {
            return Err(DaemonError::NotRunning.into());
        }
        let socket_path = constants::socket_path();
        wait_for_socket(&socket_path, 50).await?;

        let mut stream = IpcStream::connect(&socket_path).await?;
        info!("Connected to daemon at {}", socket_path.to_string_lossy());

        let serialized_command =
            serde_json::to_string(&command).map_err(|error| DaemonError::SocketError {
                reason: format!("failed to serialize command: {}", error),
            })?;
        stream.send_message(&serialized_command).await?;

        let response =
            stream
                .read_message()
                .await?
                .ok_or_else(|| DaemonError::ConnectionFailed {
                    reason: "daemon closed connection without response".to_string(),
                })?;

        if let Some(data) = response.strip_prefix("ok:") {
            Ok(data.to_string())
        } else if let Some(error) = response.strip_prefix("err:") {
            Err(DaemonError::CommandFailed {
                command: error.to_string(),
            }
            .into())
        } else {
            Err(DaemonError::InvalidResponse {
                reason: format!("missing ok:/err: prefix: {}", response),
            }
            .into())
        }
    }

    async fn is_daemon_running(&self) -> bool {
        let socket_path = constants::socket_path();

        match IpcStream::connect(&socket_path).await {
            Ok(mut stream) => {
                let health_command = DaemonCommand::HealthCheck;
                let health_message = match serde_json::to_string(&health_command) {
                    Ok(serialized) => serialized,
                    Err(_) => return false,
                };

                if stream.send_message(&health_message).await.is_err() {
                    return false;
                }

                match stream.read_message().await {
                    Ok(Some(_)) => {
                        info!("Daemon is healthy at {}", socket_path.to_string_lossy());
                        true
                    }
                    _ => {
                        info!("Daemon did not respond to health check");
                        false
                    }
                }
            }
            Err(error) => {
                info!("Daemon connection error: {:?}", error);
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

    Err(DaemonError::ConnectionFailed {
        reason: "timed out waiting for daemon socket".to_string(),
    }
    .into())
}
