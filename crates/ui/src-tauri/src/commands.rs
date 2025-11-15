use std::str::FromStr;

use byocvpn_aws::{AwsProvider, AwsProviderConfig};
use byocvpn_core::{
    self,
    cloud_provider::{CloudProvider, CloudProviderName},
    commands, credentials,
    error::{Error, Result},
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use serde_json::{Value, json};

async fn create_cloud_provider(
    cloud_provider_name: &str,
    region: Option<String>,
) -> Result<Box<dyn CloudProvider>> {
    // Get stored credentials
    let credentials = credentials::get_credentials().await?;

    // Create AWS provider config
    match cloud_provider_name {
        "aws" => {
            let config = AwsProviderConfig {
                region,
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(&config).await;
            Ok(Box::new(cloud_provider))
        }
        _ => {
            return Err(Error::InvalidCloudProviderName(
                cloud_provider_name.to_string(),
            ));
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
        Err(_) => return Err(Error::InvalidCloudProviderName(cloud_provider_name)),
    };
    credentials::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await?;
    Ok(())
}

#[tauri::command]
pub async fn verify_permissions() -> Result<serde_json::Value> {
    let cloud_provider = create_cloud_provider("aws", None).await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(region: String) -> Result<serde_json::Value> {
    // Get stored credentials
    let cloud_provider = create_cloud_provider("aws", Some(region.clone())).await?;

    // Generate keypair for this instance
    commands::setup::setup(&*cloud_provider).await?;

    commands::setup::enable_region(&*cloud_provider, &region).await?;

    let (instance_id, public_ip_v4, public_ip_v6) =
        commands::spawn::spawn_instance(&*cloud_provider).await?;

    // Return instance details
    Ok(serde_json::json!({
        "instance_id": instance_id,
        "public_ip_v4": public_ip_v4,
        "public_ip_v6": public_ip_v6,
        "region": region,
    }))
}

#[tauri::command]
pub async fn terminate_instance(instance_id: String, region: String) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &instance_id).await?;
    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: String) -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    // Terminate the instance
    let instances = commands::list::list_instances(&*cloud_provider).await?;

    Ok(instances
        .into_iter()
        .map(|instance| {
            json!({
                "id": instance.id,
                "name": instance.name,
                "state": instance.state,
                "public_ip_v4": instance.public_ip_v4,
                "public_ip_v6": instance.public_ip_v6,
            })
        })
        .collect::<Vec<Value>>())
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
pub async fn get_regions() -> Result<Vec<serde_json::Value>> {
    let cloud_provider = create_cloud_provider("aws", None).await?;

    let regions = commands::setup::get_regions(&*cloud_provider).await?;

    Ok(regions
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "country": r.country,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn connect(instance_id: String, region: String) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    let daemon_client = UnixDaemonClient;
    println!("Connecting to instance {}", instance_id.clone());
    commands::connect::connect(&*cloud_provider, &daemon_client, instance_id.clone()).await?;

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id.clone()
    ))
}

#[tauri::command]
pub async fn disconnect() -> Result<String> {
    let daemon_client = UnixDaemonClient;
    commands::disconnect::disconnect(&daemon_client).await?;
    Ok("Disconnected successfully.".to_string())
}
