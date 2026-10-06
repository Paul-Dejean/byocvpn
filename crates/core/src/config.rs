use std::{path::PathBuf, sync::OnceLock};

use handlebars::Handlebars;
use serde::Serialize;
use tokio::fs::{create_dir_all, try_exists};

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, Result},
};
use log::*;

static DATA_DIRECTORY_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Serialize)]
struct ClientConfigContext {
    client_private_key: String,
    server_public_key: String,
    server_ip_v4: String,
}

pub fn generate_client_config(
    client_private_key: &str,
    server_public_key: &str,
    server_ip_v4: &str,
) -> Result<String> {
    let template_text: &str = include_str!("templates/client_config.hbs");

    let context = ClientConfigContext {
        client_private_key: client_private_key.to_string(),
        server_public_key: server_public_key.to_string(),
        server_ip_v4: server_ip_v4.to_string(),
    };

    let handlebars_registry = Handlebars::new();

    let config = handlebars_registry
        .render_template(template_text, &context)
        .map_err(|error| ConfigurationError::TemplateRender {
            reason: error.to_string(),
        })?;
    debug!(
        "WireGuard client config generated for server IP: {}",
        server_ip_v4
    );
    Ok(config)
}

pub async fn get_wireguard_config_file_path(
    provider_name: &CloudProviderName,
    region: &str,
    instance_id: &str,
) -> Result<PathBuf> {
    let file_name = get_wireguard_config_file_name(provider_name, region, instance_id);
    let directory = get_configs_path().await?;
    let path = directory.join(file_name);
    debug!("Resolved WireGuard config file path: {}", path.display());
    Ok(path)
}

async fn get_configs_path() -> Result<PathBuf> {
    let byocvpn_dir = get_data_directory()?.join("configs");

    if !try_exists(&byocvpn_dir)
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to check configs directory: {}", error),
        })?
    {
        debug!("Creating configs directory: {}", byocvpn_dir.display());
        create_dir_all(&byocvpn_dir).await.map_err(|error| {
            ConfigurationError::TunnelConfiguration {
                reason: format!("failed to create configs directory: {}", error),
            }
        })?;
    }

    Ok(byocvpn_dir)
}

pub fn session_file_path() -> Result<PathBuf> {
    Ok(get_data_directory()?.join("session.json"))
}

fn get_wireguard_config_file_name(
    provider_name: &CloudProviderName,
    region: &str,
    instance_id: &str,
) -> String {
    let safe_id = instance_id.replace('/', "_");
    format!("{provider_name}-{region}-{safe_id}.conf")
}

pub fn set_data_directory(data_directory: PathBuf) {
    if DATA_DIRECTORY_OVERRIDE.set(data_directory).is_err() {
        warn!("Data directory was already set; keeping the existing one");
    }
}

pub fn get_data_directory() -> Result<PathBuf> {
    if let Some(data_directory) = DATA_DIRECTORY_OVERRIDE.get() {
        return Ok(data_directory.clone());
    }
    let home_dir = dirs::home_dir().ok_or(ConfigurationError::HomeDirectoryNotAvailable)?;
    Ok(home_dir.join(".byocvpn"))
}
