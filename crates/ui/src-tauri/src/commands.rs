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
    // Get stored credentials
    let credentials = credentials::get_credentials().await?;

    // Create AWS provider config
    match cloud_provider_name {
        "aws" => {
            let config = AwsProviderConfig {
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(config).await;
            Ok(Box::new(cloud_provider))
        }
        _ => {
            return Err(
                ConfigurationError::InvalidCloudProvider(cloud_provider_name.to_string()).into(),
            );
        }
    }
}

#[tauri::command]
pub async fn save_credentials(
    cloud_provider_name: String,
    access_key_id: String,
    secret_access_key: String,
) -> Result<()> {
    let cloud_provider = match CloudProviderName::from_str(&cloud_provider_name) {
        Ok(provider) => provider,
        Err(_) => return Err(ConfigurationError::InvalidCloudProvider(cloud_provider_name).into()),
    };
    credentials::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await?;
    Ok(())
}

#[tauri::command]
pub async fn verify_permissions() -> Result<Value> {
    let cloud_provider = create_cloud_provider("aws").await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(region: String) -> Result<InstanceInfo> {
    // Get stored credentials
    let cloud_provider = create_cloud_provider("aws").await?;

    // Generate keypair for this instance
    commands::setup::setup(&*cloud_provider).await?;
    commands::setup::enable_region(&*cloud_provider, &region).await?;

    let instance = commands::spawn::spawn_instance(&*cloud_provider, &region).await?;

    Ok(instance)
}

#[tauri::command]
pub async fn terminate_instance(instance_id: String, region: String) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws").await?;
    commands::terminate::terminate_instance(&*cloud_provider, &region, &instance_id).await?;
    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: Option<String>) -> Result<Vec<InstanceInfo>> {
    let cloud_provider = create_cloud_provider("aws").await?;
    let instances = commands::list::list_instances(&*cloud_provider, region.as_deref()).await?;
    Ok(instances)
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
pub async fn get_regions() -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider("aws").await?;

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

    let status: VpnStatus = serde_json::from_str(&response).map_err(|e| {
        ConfigurationError::InvalidCloudProvider(format!("Failed to parse status: {}", e))
    })?;

    Ok(status)
}

#[tauri::command]
pub async fn connect(instance_id: String, region: String, app_handle: AppHandle) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws").await?;
    let daemon_client = UnixDaemonClient;
    println!("Connecting to instance {}", instance_id.clone());
    commands::connect::connect(
        &*cloud_provider,
        &daemon_client,
        region.as_str(),
        &instance_id,
    )
    .await?;

    let vpn_status = fetch_vpn_status().await?;

    if let Some(ref connected_instance) = vpn_status.instance {
        println!("Starting metrics stream...");
        let emit_handle = app_handle.clone();
        match metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            move |status| {
                let _ = emit_handle.emit("vpn-status", &status);
            },
        )
        .await
        {
            Ok(_) => println!("Started metrics stream"),
            Err(error) => eprintln!("Failed to start metrics stream: {}", error),
        }
    }

    let _ = app_handle.emit(
        "vpn-status",
        &VpnStatus {
            connected: vpn_status.connected,
            instance: vpn_status.instance,
            metrics: None,
        },
    );

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
    let status = fetch_vpn_status().await?;
    Ok(VpnStatus {
        connected: status.connected,
        instance: status.instance,
        metrics: None,
    })
}

#[tauri::command]
pub async fn start_metrics_stream(app_handle: AppHandle) -> Result<()> {
    let status = fetch_vpn_status().await?;
    match status.instance {
        Some(instance) => {
            metrics_stream::start(
                byocvpn_daemon::constants::metrics_socket_path(),
                instance,
                move |vpn_status| {
                    let _ = app_handle.emit("vpn-status", &vpn_status);
                },
            )
            .await
        }
        None => Err(ConfigurationError::InvalidCloudProvider(
            "Cannot start metrics stream: not connected to VPN".to_string(),
        )
        .into()),
    }
}
