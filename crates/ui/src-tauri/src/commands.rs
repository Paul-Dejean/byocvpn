use std::str::FromStr;

use byocvpn_aws::{AwsProvider, provider::AwsProviderConfig};
use byocvpn_core::{
    self,
    cloud_provider::{CloudProvider, CloudProviderName},
    commands,
};
use serde_json::{Value, json};

async fn create_cloud_provider(
    cloud_provider_name: &str,
    region: Option<String>,
) -> Result<Box<dyn CloudProvider>, Box<dyn std::error::Error>> {
    // Get stored credentials
    let credentials = byocvpn_core::get_credentials().await?;
    println!("Credentials fetched: {:?}", credentials);

    // Create AWS provider config
    match cloud_provider_name {
        "aws" => {
            let config = AwsProviderConfig {
                region,
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(&config).await?;

            Ok(Box::new(cloud_provider))
        }
        _ => return Err("Unsupported cloud provider".into()),
    }
}

#[tauri::command]
pub async fn save_credentials(
    cloud_provider_name: String,
    access_key_id: String,
    secret_access_key: String,
) -> Result<String, String> {
    let cloud_provider = match CloudProviderName::from_str(&cloud_provider_name) {
        Ok(provider) => provider,
        Err(_) => return Err("Invalid cloud provider type".to_string()),
    };

    let res =
        byocvpn_core::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await;
    return res;
}

#[tauri::command]
pub async fn verify_permissions() -> Result<serde_json::Value, String> {
    let cloud_provider = create_cloud_provider("aws", None)
        .await
        .map_err(|e| e.to_string())?;

    let result = byocvpn_core::verify_permissions(&*cloud_provider)
        .await
        .map_err(|e| e.to_string());

    return result;
}

#[tauri::command]
pub async fn spawn_instance(region: String) -> Result<serde_json::Value, String> {
    // Get stored credentials
    let cloud_provider = create_cloud_provider("aws", Some(region.clone()))
        .await
        .map_err(|e| e.to_string())?;

    // Generate keypair for this instance
    commands::setup::setup(&*cloud_provider)
        .await
        .map_err(|e| e.to_string())?;

    commands::setup::enable_region(&*cloud_provider, &region)
        .await
        .map_err(|e| e.to_string())?;

    let (instance_id, public_ip_v4, public_ip_v6) =
        commands::spawn::spawn_instance(&*cloud_provider)
            .await
            .map_err(|e| e.to_string())?;

    // Return instance details
    Ok(serde_json::json!({
        "instance_id": instance_id,
        "public_ip_v4": public_ip_v4,
        "public_ip_v6": public_ip_v6,
        "region": region,
    }))
}

#[tauri::command]
pub async fn terminate_instance(instance_id: String, region: String) -> Result<String, String> {
    // Get stored credentials

    let cloud_provider = create_cloud_provider("aws", Some(region))
        .await
        .map_err(|e| e.to_string())?;
    // Terminate the instance
    commands::terminate::terminate_instance(&*cloud_provider, &instance_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: String) -> Result<Vec<Value>, String> {
    let cloud_provider = create_cloud_provider("aws", Some(region))
        .await
        .map_err(|e| e.to_string())?;
    // Terminate the instance
    let instances = commands::list::list_instances(&*cloud_provider)
        .await
        .map_err(|e| e.to_string())?;

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
pub async fn has_profile() -> Result<bool, String> {
    // Try to get credentials - if they exist and are valid, return true
    match byocvpn_core::get_credentials().await {
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
pub async fn get_regions() -> Result<Vec<serde_json::Value>, String> {
    let cloud_provider = create_cloud_provider("aws", None)
        .await
        .map_err(|e| e.to_string())?;

    let regions = commands::setup::get_regions(&*cloud_provider)
        .await
        .map_err(|e| e.to_string())?;

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
pub async fn connect(instance_id: String, region: String) -> Result<String, String> {
    let cloud_provider = create_cloud_provider("aws", Some(region))
        .await
        .map_err(|e| e.to_string())?;

    commands::connect::connect(&*cloud_provider, instance_id.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id.clone()
    ))
}
