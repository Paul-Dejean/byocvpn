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
    let service_name = if cfg!(debug_assertions) {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    Path::new(&format!("/etc/systemd/system/{}.service", service_name)).exists()
}

#[cfg(all(target_os = "linux", not(feature = "external-daemon")))]
pub fn install_daemon() -> Result<()> {
    let is_dev = cfg!(debug_assertions);
    let service_name = if is_dev {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let installed_binary_name = service_name;
    let build_dir = if is_dev { "debug" } else { "release" };

    let current_executable_path =
        std::env::current_exe().map_err(|error| ConfigurationError::InvalidFile {
            reason: error.to_string(),
        })?;

    let workspace_root = current_executable_path
        .ancestors()
        .find(|path| path.join("Cargo.toml").exists());

    let exe_dir =
        current_executable_path
            .parent()
            .ok_or_else(|| ConfigurationError::FileNotFound {
                path: "executable directory".to_string(),
            })?;

    let daemon_binary_path = [
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

    let installed_binary_path = format!("/usr/local/bin/{}", installed_binary_name);
    let service_file_path = format!("/etc/systemd/system/{}.service", service_name);
    let systemd_unit_content = format!(
        "[Unit]\nDescription=byocvpn Daemon\nAfter=network.target\n\n[Service]\nType=simple\nExecStart={installed_binary_path}\nRestart=always\nRestartSec=5\n\n[Install]\nWantedBy=multi-user.target\n",
        installed_binary_path = installed_binary_path,
    );

    let daemon_binary_src = daemon_binary_path.display().to_string();

    let script_content = format!(
        "cp '{src}' '{dst}'\nchmod 755 '{dst}'\nprintf '%s' '{unit}' > '{service}'\nsystemctl daemon-reload\nsystemctl enable {service_name}\nsystemctl start {service_name}\n",
        src = daemon_binary_src,
        dst = installed_binary_path,
        unit = systemd_unit_content.replace('\'', "'\\''"),
        service = service_file_path,
        service_name = service_name,
    );

    let temp_script = std::env::temp_dir().join("byocvpn_install.sh");
    std::fs::write(&temp_script, &script_content).map_err(|error| {
        ConfigurationError::InvalidFile {
            reason: error.to_string(),
        }
    })?;

    let output = std::process::Command::new("pkexec")
        .args(["bash", &temp_script.display().to_string()])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile {
            reason: error.to_string(),
        })?;

    let _ = std::fs::remove_file(&temp_script);

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(ConfigurationError::InvalidFile {
            reason: format!("daemon installation failed: {}", detail.trim()),
        }
        .into())
    }
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
    let service_name = if cfg!(debug_assertions) {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let script_content = format!(
        "systemctl stop {service_name}\nsystemctl disable {service_name}\nrm -f /etc/systemd/system/{service_name}.service\nrm -f /usr/local/bin/{service_name}\nsystemctl daemon-reload\n",
        service_name = service_name,
    );
    let script_content = script_content.as_str();

    let temp_script = std::env::temp_dir().join("byocvpn_uninstall.sh");
    std::fs::write(&temp_script, script_content).map_err(|error| {
        ConfigurationError::InvalidFile {
            reason: error.to_string(),
        }
    })?;

    let output = std::process::Command::new("pkexec")
        .args(["bash", &temp_script.display().to_string()])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile {
            reason: error.to_string(),
        })?;

    let _ = std::fs::remove_file(&temp_script);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(ConfigurationError::InvalidFile {
            reason: format!("daemon uninstall failed: {}", detail.trim()),
        }
        .into());
    }

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
