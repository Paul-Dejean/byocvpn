use async_trait::async_trait;
use byocvpn_core::{
    daemon_client::{DaemonClient, DaemonCommand, DaemonResponse},
    error::{DaemonError, Result},
    ipc::IpcStream,
};
use serde_json::Value;
use tokio::time::{Duration, sleep};

use crate::constants;
use log::*;

pub struct UnixDaemonClient;

#[async_trait]
impl DaemonClient for UnixDaemonClient {
    async fn send_command(&self, command: DaemonCommand) -> Result<Value> {
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

        let raw_response =
            stream
                .read_message()
                .await?
                .ok_or_else(|| DaemonError::ConnectionFailed {
                    reason: "daemon closed connection without response".to_string(),
                })?;

        let response = serde_json::from_str::<DaemonResponse>(&raw_response).map_err(|error| {
            DaemonError::InvalidResponse {
                reason: format!("failed to parse daemon response: {}", error),
            }
        })?;

        match response {
            DaemonResponse::Ok(value) => Ok(value),
            DaemonResponse::Err(error) => Err(error.into()),
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
