use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigurationError, Result};

#[cfg(target_os = "macos")]
pub fn is_daemon_installed() -> bool {
    let binary_name = if cfg!(debug_assertions) { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
    let label = if cfg!(debug_assertions) { "com.byocvpn.daemon.dev" } else { "com.byocvpn.daemon" };
    Path::new(&format!("/Library/PrivilegedHelperTools/{}", binary_name)).exists()
        && Path::new(&format!("/Library/LaunchDaemons/{}.plist", label)).exists()
}

#[cfg(target_os = "macos")]
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

#[cfg(windows)]
pub fn is_daemon_installed() -> bool {
    let service_name =
        if cfg!(debug_assertions) { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
    matches!(
        std::process::Command::new("sc").args(["query", service_name]).output(),
        Ok(output) if output.status.success()
    )
}

#[cfg(windows)]
pub fn install_daemon() -> Result<()> {
    let is_dev = cfg!(debug_assertions);
    let service_name = if is_dev { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
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
        exe_dir.join("byocvpn_daemon.exe"),
        exe_dir.join("byocvpn-daemon.exe"),
        workspace_root
            .map(|root| root.join("target").join(build_dir).join("byocvpn_daemon.exe"))
            .unwrap_or_default(),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or_else(|| ConfigurationError::FileNotFound {
        path: format!("target/{}/byocvpn_daemon.exe", build_dir),
    })?;

    let install_dir = r"C:\Program Files\byocvpn";
    let installed_binary = format!(r"{}\byocvpn-daemon.exe", install_dir);
    let daemon_src = daemon_binary_path.display().to_string();

    let script_content = format!(
        "New-Item -ItemType Directory -Force -Path '{install_dir}'\r\nCopy-Item -Path '{src}' -Destination '{dst}' -Force\r\nsc.exe create {service_name} binPath= '{dst} --service' start= auto DisplayName= 'byocvpn Daemon'\r\nsc.exe start {service_name}\r\n",
        install_dir = install_dir,
        src = daemon_src,
        dst = installed_binary,
        service_name = service_name,
    );

    let temp_script = std::env::temp_dir().join("byocvpn_install.ps1");
    std::fs::write(&temp_script, &script_content)
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -Wait",
                temp_script.display()
            ),
        ])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

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

#[cfg(target_os = "linux")]
pub fn is_daemon_installed() -> bool {
    Path::new("/etc/systemd/system/byocvpn-daemon.service").exists()
}

#[cfg(target_os = "linux")]
pub fn install_daemon() -> Result<()> {
    let build_dir = if cfg!(debug_assertions) { "debug" } else { "release" };

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
        exe_dir.join("byocvpn-daemon"),
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

    let systemd_unit_content = "[Unit]\nDescription=byocvpn Daemon\nAfter=network.target\n\n[Service]\nType=simple\nExecStart=/usr/local/bin/byocvpn-daemon\nRestart=always\nRestartSec=5\n\n[Install]\nWantedBy=multi-user.target\n";

    let daemon_binary_src = daemon_binary_path.display().to_string();

    let script_content = format!(
        "cp '{src}' /usr/local/bin/byocvpn-daemon\nchmod 755 /usr/local/bin/byocvpn-daemon\nprintf '%s' '{unit}' > /etc/systemd/system/byocvpn-daemon.service\nsystemctl daemon-reload\nsystemctl enable byocvpn-daemon\nsystemctl start byocvpn-daemon\n",
        src = daemon_binary_src,
        unit = systemd_unit_content.replace('\'', "'\\''"),
    );

    let temp_script = std::env::temp_dir().join("byocvpn_install.sh");
    std::fs::write(&temp_script, &script_content)
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    let output = std::process::Command::new("pkexec")
        .args(["bash", &temp_script.display().to_string()])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

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

#[cfg(target_os = "macos")]
pub fn uninstall_daemon() -> Result<()> {
    let binary_name = if cfg!(debug_assertions) { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };
    let label = if cfg!(debug_assertions) { "com.byocvpn.daemon.dev" } else { "com.byocvpn.daemon" };

    let script = format!(
        r#"do shell script "
            launchctl unload '/Library/LaunchDaemons/{label}.plist' 2>/dev/null; \
            rm -f '/Library/PrivilegedHelperTools/{binary_name}' && \
            rm -f '/Library/LaunchDaemons/{label}.plist'
        " with administrator privileges"#,
        label = label,
        binary_name = binary_name,
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(ConfigurationError::InvalidFile {
            reason: format!("osascript failed: {}", detail.trim()),
        }
        .into());
    }

    remove_user_data()
}

#[cfg(windows)]
pub fn uninstall_daemon() -> Result<()> {
    let service_name =
        if cfg!(debug_assertions) { "byocvpn-daemon-dev" } else { "byocvpn-daemon" };

    let script_content = format!(
        "sc.exe stop {service_name}\r\nsc.exe delete {service_name}\r\nRemove-Item -Force -ErrorAction SilentlyContinue 'C:\\Program Files\\byocvpn\\byocvpn-daemon.exe'\r\n",
        service_name = service_name,
    );

    let temp_script = std::env::temp_dir().join("byocvpn_uninstall.ps1");
    std::fs::write(&temp_script, &script_content)
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -Wait",
                temp_script.display()
            ),
        ])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

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

#[cfg(target_os = "linux")]
pub fn uninstall_daemon() -> Result<()> {
    let script_content = "systemctl stop byocvpn-daemon\nsystemctl disable byocvpn-daemon\nrm -f /etc/systemd/system/byocvpn-daemon.service\nrm -f /usr/local/bin/byocvpn-daemon\nsystemctl daemon-reload\n";

    let temp_script = std::env::temp_dir().join("byocvpn_uninstall.sh");
    std::fs::write(&temp_script, script_content)
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

    let output = std::process::Command::new("pkexec")
        .args(["bash", &temp_script.display().to_string()])
        .output()
        .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;

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
    let home_dir = dirs::home_dir()
        .ok_or_else(|| ConfigurationError::HomeDirectoryNotAvailable)?;
    let byocvpn_dir = home_dir.join(".byocvpn");
    if byocvpn_dir.exists() {
        std::fs::remove_dir_all(&byocvpn_dir)
            .map_err(|error| ConfigurationError::InvalidFile { reason: error.to_string() })?;
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
