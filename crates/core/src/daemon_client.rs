use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonCommand {
    Connect {
        config_path: String,
        region: String,
        provider: String,
        public_ip_v4: Option<String>,
        public_ip_v6: Option<String>,
    },
    Disconnect,
    Status,
    Stats,
    HealthCheck,
}
#[async_trait]
pub trait DaemonClient: Send + Sync {
    async fn send_command(&self, cmd: DaemonCommand) -> Result<String>;
    async fn is_daemon_running(&self) -> bool;
}
