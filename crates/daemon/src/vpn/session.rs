use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, Result},
};
use log::*;
use serde::{Deserialize, Serialize};

fn default_kill_switch_enabled() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedSession {
    pub instance_id: String,
    pub provider: CloudProviderName,
    pub region: String,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
    #[serde(default = "default_kill_switch_enabled")]
    pub kill_switch_enabled: bool,
}

pub fn write_session(session: &PersistedSession) -> Result<()> {
    let path = byocvpn_core::config::session_file_path()?;
    let json = serde_json::to_string(session).map_err(|error| ConfigurationError::TunnelConfiguration {
        reason: format!("failed to serialize session: {}", error),
    })?;
    std::fs::write(&path, json.as_bytes()).map_err(|error| ConfigurationError::TunnelConfiguration {
        reason: format!("failed to write session file: {}", error),
    })?;
    debug!("Session persisted to {}", path.display());
    Ok(())
}

pub fn clear_session() -> Result<()> {
    let path = byocvpn_core::config::session_file_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => {
            debug!("Session file cleared");
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ConfigurationError::TunnelConfiguration {
            reason: format!("failed to remove session file: {}", error),
        }
        .into()),
    }
}

pub fn read_session() -> Option<PersistedSession> {
    let path = byocvpn_core::config::session_file_path().ok()?;
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}
