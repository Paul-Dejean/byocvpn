use std::net::SocketAddr;

use async_trait::async_trait;
use ipnet::IpNet;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, DaemonError, Result},
    ipc::{IpcStream, socket_path},
    tunnel::VpnStatus,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct VpnConnectParams {
    pub instance_id: String,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub server_endpoint: SocketAddr,
    pub private_ipv4: IpNet,
    pub private_ipv6: IpNet,
    pub dns_servers: Vec<String>,
    pub region: String,
    pub provider: CloudProviderName,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
    pub kill_switch_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonCommand {
    Connect(Box<VpnConnectParams>),
    Disconnect,
    Status,
    Stats,
    HealthCheck,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "payload", rename_all = "snake_case")]
pub enum DaemonResponse {
    Ok(Value),
    Err(DaemonError),
}

#[async_trait]
pub trait DaemonClient: Send + Sync {
    async fn send_command(&self, command: DaemonCommand) -> Result<Value>;
    async fn is_daemon_running(&self) -> bool;
}

pub async fn fetch_vpn_status(client: &dyn DaemonClient) -> Result<VpnStatus> {
    if !client.is_daemon_running().await {
        return Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
            connected_at: None,
            connection_error: None,
        });
    }

    let response = client.send_command(DaemonCommand::Status).await?;
    serde_json::from_value(response).map_err(|error| {
        ConfigurationError::ParseError {
            value: "daemon status response".to_string(),
            reason: error.to_string(),
        }
        .into()
    })
}

pub struct IpcDaemonClient;

#[async_trait]
impl DaemonClient for IpcDaemonClient {
    async fn send_command(&self, command: DaemonCommand) -> Result<Value> {
        if !self.is_daemon_running().await {
            return Err(DaemonError::NotRunning.into());
        }
        let daemon_socket_path = socket_path();
        let mut stream = IpcStream::connect(&daemon_socket_path).await?;
        info!(
            "Connected to daemon at {}",
            daemon_socket_path.to_string_lossy()
        );

        let serialized_command =
            serde_json::to_string(&command).map_err(|error| DaemonError::SocketError {
                reason: format!("failed to serialize command: {error}"),
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
                reason: format!("failed to parse daemon response: {error}"),
            }
        })?;

        match response {
            DaemonResponse::Ok(value) => Ok(value),
            DaemonResponse::Err(error) => Err(error.into()),
        }
    }

    async fn is_daemon_running(&self) -> bool {
        let daemon_socket_path = socket_path();

        match IpcStream::connect(&daemon_socket_path).await {
            Ok(mut stream) => {
                let health_command = DaemonCommand::HealthCheck;
                let Ok(health_message) = serde_json::to_string(&health_command) else {
                    return false;
                };

                if stream.send_message(&health_message).await.is_err() {
                    return false;
                }

                if let Ok(Some(_)) = stream.read_message().await {
                    info!(
                        "Daemon is healthy at {}",
                        daemon_socket_path.to_string_lossy()
                    );
                    true
                } else {
                    debug!("Daemon did not respond to health check");
                    false
                }
            }
            Err(error) => {
                debug!("Daemon connection error: {error:?}");
                false
            }
        }
    }
}
