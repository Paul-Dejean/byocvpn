use std::net::SocketAddr;

use async_trait::async_trait;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    cloud_provider::CloudProviderName,
    error::{DaemonError, Result},
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonCommand {
    Connect(VpnConnectParams),
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
