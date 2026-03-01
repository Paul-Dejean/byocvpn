use std::str::FromStr;

use byocvpn_aws::{AwsProvider, AwsProviderConfig};
use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName, InstanceInfo},
    commands, credentials,
    daemon_client::{DaemonClient, DaemonCommand},
    error::{ConfigurationError, Result},
    metrics_stream,
    tunnel::VpnStatus,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

async fn create_cloud_provider(cloud_provider_name: &str) -> Result<Box<dyn CloudProvider>> {
    match cloud_provider_name {
        "aws" => {
            let credentials = credentials::get_credentials().await?;
            let config = AwsProviderConfig {
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(config).await;
            Ok(Box::new(cloud_provider) as Box<dyn CloudProvider>)
        }
        "oracle" => {
            let oracle_credentials = credentials::get_oracle_credentials().await?;
            let config = byocvpn_oracle::OracleProviderConfig {
                tenancy_ocid: oracle_credentials.tenancy_ocid,
                user_ocid: oracle_credentials.user_ocid,
                fingerprint: oracle_credentials.fingerprint,
                private_key_pem: oracle_credentials.private_key_pem,
                region: oracle_credentials.region,
            };
            Ok(Box::new(byocvpn_oracle::OracleProvider::new(config)) as Box<dyn CloudProvider>)
        }
        _ => Err(ConfigurationError::InvalidCloudProvider(cloud_provider_name.to_string()).into()),
    }
}

#[tauri::command]
pub async fn get_credentials(provider: String) -> Result<Value> {
    match provider.as_str() {
        "aws" => match credentials::get_credentials().await {
            Ok(creds) => Ok(json!({
                "accessKeyId": creds.access_key,
                "secretAccessKey": creds.secret_access_key,
            })),
            Err(_) => Ok(json!(null)),
        },
        "oracle" => match credentials::get_oracle_credentials().await {
            Ok(creds) => Ok(json!({
                "tenancyOcid": creds.tenancy_ocid,
                "userOcid": creds.user_ocid,
                "fingerprint": creds.fingerprint,
                "privateKeyPem": creds.private_key_pem,
                "region": creds.region,
            })),
            Err(_) => Ok(json!(null)),
        },
        _ => Err(ConfigurationError::InvalidCloudProvider(provider).into()),
    }
}

#[tauri::command]
pub async fn save_credentials(provider: String, creds: Value) -> Result<()> {
    match provider.as_str() {
        "aws" => {
            let access_key_id = creds["accessKeyId"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing accessKeyId".into())
                })?
                .to_string();
            let secret_access_key = creds["secretAccessKey"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing secretAccessKey".into())
                })?
                .to_string();
            let cloud_provider = CloudProviderName::from_str(&provider)
                .map_err(|_| ConfigurationError::InvalidCloudProvider(provider.clone()))?;
            credentials::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await
        }
        "oracle" => {
            let tenancy_ocid = creds["tenancyOcid"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing tenancyOcid".into())
                })?
                .to_string();
            let user_ocid = creds["userOcid"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing userOcid".into()))?
                .to_string();
            let fingerprint = creds["fingerprint"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing fingerprint".into())
                })?
                .to_string();
            let private_key_pem = creds["privateKeyPem"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing privateKeyPem".into())
                })?
                .to_string();
            let region = creds["region"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing region".into()))?
                .to_string();
            credentials::save_oracle_credentials(
                &tenancy_ocid,
                &user_ocid,
                &fingerprint,
                &private_key_pem,
                &region,
            )
            .await
        }
        _ => Err(ConfigurationError::InvalidCloudProvider(provider).into()),
    }
}

#[tauri::command]
pub async fn verify_permissions() -> Result<Value> {
    let cloud_provider = create_cloud_provider("aws").await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(region: String, provider: String) -> Result<InstanceInfo> {
    let cloud_provider = create_cloud_provider(&provider).await?;

    commands::setup::setup(&*cloud_provider).await?;
    commands::setup::enable_region(&*cloud_provider, &region).await?;

    let instance = commands::spawn::spawn_instance(&*cloud_provider, &region).await?;

    Ok(instance)
}

#[tauri::command]
pub async fn terminate_instance(
    instance_id: String,
    region: String,
    provider: String,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(&provider).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &region, &instance_id).await?;
    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: Option<String>) -> Result<Vec<InstanceInfo>> {
    let mut all_instances: Vec<InstanceInfo> = Vec::new();

    for provider_name in &["aws", "oracle"] {
        match create_cloud_provider(provider_name).await {
            Ok(provider) => {
                match commands::list::list_instances(&*provider, region.as_deref()).await {
                    Ok(instances) => all_instances.extend(instances),
                    Err(error) => {
                        eprintln!("Failed to list {} instances: {}", provider_name, error)
                    }
                }
            }
            Err(_) => {} // Provider not configured; skip silently.
        }
    }

    Ok(all_instances)
}

#[tauri::command]
pub async fn has_profile() -> Result<bool> {
    // Try to get credentials - if they exist and are valid, return true
    match credentials::get_credentials().await {
        Ok(credentials) => {
            // Check if credentials are not empty
            if !credentials.access_key.is_empty() && !credentials.secret_access_key.is_empty() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => {
            // Credentials file doesn't exist or can't be read
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_regions(provider: String) -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider(&provider).await?;

    let regions = commands::setup::get_regions(&*cloud_provider).await?;

    Ok(regions
        .into_iter()
        .map(|r| {
            json!({
                "name": r.name,
                "country": r.country,
            })
        })
        .collect())
}

async fn fetch_vpn_status() -> Result<VpnStatus> {
    let daemon_client = UnixDaemonClient;

    if !daemon_client.is_daemon_running().await {
        return Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
        });
    }

    let response = daemon_client.send_command(DaemonCommand::Status).await?;

    serde_json::from_str(&response).map_err(|error| {
        ConfigurationError::InvalidCloudProvider(format!("Failed to parse status: {}", error))
            .into()
    })
}

#[tauri::command]
pub async fn connect(
    instance_id: String,
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(&provider).await?;
    let daemon_client = UnixDaemonClient;

    commands::connect::connect(
        &*cloud_provider,
        &daemon_client,
        region.as_str(),
        &instance_id,
    )
    .await?;

    let vpn_status = fetch_vpn_status().await?;

    if let Some(ref connected_instance) = vpn_status.instance {
        let emit_handle = app_handle.clone();
        if let Err(error) = metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            move |status| {
                let _ = emit_handle.emit("vpn-status", &status);
            },
        )
        .await
        {
            eprintln!("Failed to start metrics stream: {}", error);
        }
    }

    let _ = app_handle.emit("vpn-status", &vpn_status);

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id
    ))
}

#[tauri::command]
pub async fn disconnect(app_handle: AppHandle) -> Result<String> {
    metrics_stream::stop().await?;

    let daemon_client = UnixDaemonClient;
    commands::disconnect::disconnect(&daemon_client).await?;

    let _ = app_handle.emit(
        "vpn-status",
        &VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
        },
    );

    Ok("Disconnected successfully.".to_string())
}

#[tauri::command]
pub async fn get_vpn_status() -> Result<VpnStatus> {
    fetch_vpn_status().await
}

#[tauri::command]
pub async fn subscribe_to_vpn_status(app_handle: AppHandle) -> Result<()> {
    let status = fetch_vpn_status().await?;
    match status.instance {
        Some(connected_instance) => {
            metrics_stream::start(
                byocvpn_daemon::constants::metrics_socket_path(),
                connected_instance,
                move |vpn_status| {
                    let _ = app_handle.emit("vpn-status", &vpn_status);
                },
            )
            .await
        }
        None => Err(ConfigurationError::InvalidCloudProvider(
            "Cannot subscribe to VPN status: not connected to VPN".to_string(),
        )
        .into()),
    }
}

#[tauri::command]
pub async fn is_daemon_installed() -> Result<bool> {
    let installed_binary_name = if cfg!(debug_assertions) {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let label = if cfg!(debug_assertions) {
        "com.byocvpn.daemon.dev"
    } else {
        "com.byocvpn.daemon"
    };
    Ok(std::path::Path::new(&format!(
        "/Library/PrivilegedHelperTools/{}",
        installed_binary_name
    ))
    .exists()
        && std::path::Path::new(&format!("/Library/LaunchDaemons/{}.plist", label)).exists())
}

#[tauri::command]
pub async fn install_daemon() -> Result<()> {
    // In debug mode install the dev daemon; in release install the production daemon.
    let is_dev = cfg!(debug_assertions);

    let installed_binary_name = if is_dev {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let plist_name = if is_dev {
        "com.byocvpn.daemon.dev.plist"
    } else {
        "com.byocvpn.daemon.plist"
    };
    let label = if is_dev {
        "com.byocvpn.daemon.dev"
    } else {
        "com.byocvpn.daemon"
    };
    let build_dir = if is_dev { "debug" } else { "release" };

    let current_executable_path = std::env::current_exe()
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;

    let workspace_root = current_executable_path
        .ancestors()
        .find(|path| path.join("Cargo.toml").exists());

    let exe_dir = current_executable_path.parent().ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider("Could not determine exe directory".to_string())
    })?;

    // Look for the daemon binary: bundled Resources/ first, then alongside exe, then workspace target/
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
    .ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider(format!(
            "Daemon binary not found in target/{}/",
            build_dir
        ))
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
    .ok_or_else(|| ConfigurationError::InvalidCloudProvider(format!("{} not found", plist_name)))?;

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
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "osascript failed: {}",
            detail.trim()
        ))
        .into())
    }
}
