#[cfg(not(feature = "external-daemon"))]
use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigurationError, Result};

#[cfg(feature = "external-daemon")]
pub fn is_daemon_installed() -> bool {
    true
}

#[cfg(feature = "external-daemon")]
pub fn install_daemon() -> Result<()> {
    Ok(())
}

#[cfg(feature = "external-daemon")]
pub fn uninstall_daemon() -> Result<()> {
    remove_user_data()
}

#[cfg(all(target_os = "macos", not(feature = "external-daemon")))]
pub fn is_daemon_installed() -> bool {
    #[cfg(not(debug_assertions))]
    {
        return true;
    }
    #[cfg(debug_assertions)]
    {
        Path::new("/Library/PrivilegedHelperTools/byocvpn-daemon-dev").exists()
            && Path::new("/Library/LaunchDaemons/com.byocvpn.daemon.dev.plist").exists()
    }
}

#[cfg(all(target_os = "macos", not(feature = "external-daemon")))]
pub fn install_daemon() -> Result<()> {
    Ok(())
}

#[cfg(all(windows, not(feature = "external-daemon")))]
pub fn is_daemon_installed() -> bool {
    #[cfg(not(debug_assertions))]
    {
        return true;
    }
    #[cfg(debug_assertions)]
    {
        matches!(
            std::process::Command::new("sc")
                .args(["query", "byocvpn-daemon-dev"])
                .output(),
            Ok(output) if output.status.success()
        )
    }
}

#[cfg(all(windows, not(feature = "external-daemon")))]
pub fn install_daemon() -> Result<()> {
    Ok(())
}

#[cfg(all(target_os = "linux", not(feature = "external-daemon")))]
pub fn is_daemon_installed() -> bool {
    #[cfg(not(debug_assertions))]
    {
        return true;
    }
    #[cfg(debug_assertions)]
    {
        Path::new("/etc/systemd/system/byocvpn-daemon-dev.service").exists()
    }
}

#[cfg(all(target_os = "linux", not(feature = "external-daemon")))]
pub fn install_daemon() -> Result<()> {
    Ok(())
}

#[cfg(all(target_os = "macos", not(feature = "external-daemon")))]
pub fn uninstall_daemon() -> Result<()> {
    remove_user_data()
}

#[cfg(all(windows, not(feature = "external-daemon")))]
pub fn uninstall_daemon() -> Result<()> {
    remove_user_data()
}

#[cfg(all(target_os = "linux", not(feature = "external-daemon")))]
pub fn uninstall_daemon() -> Result<()> {
    remove_user_data()
}

fn remove_user_data() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| ConfigurationError::HomeDirectoryNotAvailable)?;
    let byocvpn_dir = home_dir.join(".byocvpn");
    if byocvpn_dir.exists() {
        std::fs::remove_dir_all(&byocvpn_dir).map_err(|error| ConfigurationError::InvalidFile {
            reason: error.to_string(),
        })?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonCommand {
    Connect { config_path: String },
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
