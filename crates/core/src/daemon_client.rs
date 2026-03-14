use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigurationError, Result};

pub fn is_daemon_installed() -> bool {
    let binary_name = if cfg!(debug_assertions) { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
    let label = if cfg!(debug_assertions) { "com.byocvpn.daemon.dev" } else { "com.byocvpn.daemon" };
    Path::new(&format!("/Library/PrivilegedHelperTools/{}", binary_name)).exists()
        && Path::new(&format!("/Library/LaunchDaemons/{}.plist", label)).exists()
}

pub fn install_daemon() -> Result<()> {
    let is_dev = cfg!(debug_assertions);

    let installed_binary_name = if is_dev { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
    let plist_name = if is_dev { "com.byocvpn.daemon.dev.plist" } else { "com.byocvpn.daemon.plist" };
    let label = if is_dev { "com.byocvpn.daemon.dev" } else { "com.byocvpn.daemon" };
    let build_dir = if is_dev { "debug" } else { "release" };

    let current_executable_path = std::env::current_exe()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    let workspace_root = current_executable_path
        .ancestors()
        .find(|path| path.join("Cargo.toml").exists());

    let exe_dir = current_executable_path
        .parent()
        .ok_or_else(|| ConfigurationError::FileNotFound {
            path: "executable directory".to_string(),
        })?;

    let daemon_binary_path = [
        exe_dir
            .parent()
            .map(|p| p.join("Resources").join("byocvpn-daemon"))
            .unwrap_or_default(),
        exe_dir.join(installed_binary_name),
        exe_dir.join("byocvpn_daemon"),
        workspace_root
            .map(|root| root.join("target").join(build_dir).join("byocvpn_daemon"))
            .unwrap_or_default(),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or_else(|| ConfigurationError::FileNotFound {
        path: format!("target/{}/byocvpn_daemon", build_dir),
    })?;

    let daemon_plist_path = [
        exe_dir
            .parent()
            .map(|p| p.join("Resources").join(plist_name))
            .unwrap_or_default(),
        workspace_root
            .map(|root| root.join("scripts").join(plist_name))
            .unwrap_or_default(),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or_else(|| ConfigurationError::FileNotFound { path: plist_name.to_string() })?;

    let script = format!(
        r#"do shell script "
            launchctl unload '/Library/LaunchDaemons/{label}.plist' 2>/dev/null; \
            cp '{}' '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            chmod 544 '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            chown root:wheel '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            cp '{}' '/Library/LaunchDaemons/{label}.plist' && \
            chmod 644 '/Library/LaunchDaemons/{label}.plist' && \
            chown root:wheel '/Library/LaunchDaemons/{label}.plist' && \
            launchctl load '/Library/LaunchDaemons/{label}.plist'
        " with administrator privileges"#,
        daemon_binary_path.display(),
        daemon_plist_path.display(),
        label = label,
        installed_binary_name = installed_binary_name,
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(ConfigurationError::InvalidFile {
            reason: format!("osascript failed: {}", detail.trim()),
        }
        .into())
    }
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
