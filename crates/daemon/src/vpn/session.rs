use std::{
    fs::{read_to_string, remove_file, write},
    io::ErrorKind,
};

use byocvpn_core::{
    cloud_provider::CloudProviderName,
    config::session_file_path,
    error::{ConfigurationError, Result},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const fn default_kill_switch_enabled() -> bool {
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
    let path = session_file_path()?;
    let json = serde_json::to_string(session)?;
    write(&path, json.as_bytes()).map_err(|source| ConfigurationError::DataFile {
        operation: "write",
        path: path.clone(),
        source,
    })?;
    debug!("session persisted to {}", path.display());
    Ok(())
}

pub fn clear_session() -> Result<()> {
    let path = session_file_path()?;
    match remove_file(&path) {
        Ok(()) => {
            debug!("session file cleared");
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(source) => Err(ConfigurationError::DataFile {
            operation: "remove",
            path,
            source,
        }
        .into()),
    }
}

pub fn read_session() -> Option<PersistedSession> {
    let path = session_file_path().ok()?;
    let json = match read_to_string(&path) {
        Ok(json) => json,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            warn!(%error, "failed to read session file {}", path.display());
            return None;
        }
    };
    match serde_json::from_str(&json) {
        Ok(session) => Some(session),
        Err(error) => {
            warn!(%error, "ignoring unreadable session file {}",
                path.display()
            );
            None
        }
    }
}
