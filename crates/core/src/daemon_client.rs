use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd")]
pub enum DaemonCommand {
    Connect { config_path: String },
    Disconnect,
    Status,
}
#[async_trait]
pub trait DaemonClient: Send + Sync {
    async fn send_command(&self, cmd: DaemonCommand) -> Result<String>;
    async fn is_daemon_running(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatus {
    pub connection_state: ConnectionState,
    pub interface_name: Option<String>,
    pub endpoint_address: Option<String>,
    pub last_error_message: Option<String>,
    pub bytes_transferred_upload: u64,
    pub bytes_transferred_download: u64,
}
