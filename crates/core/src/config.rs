use std::path::PathBuf;

use handlebars::Handlebars;
use serde::Serialize;
use tokio::fs::{create_dir_all, try_exists};

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, Result},
};
use log::*;

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
    info!("{}", &config);
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
    Ok(path)
}

async fn get_configs_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(ConfigurationError::HomeDirectoryNotAvailable)?;
    let byocvpn_dir = home_dir.join(".byocvpn").join("configs");

    if !try_exists(&byocvpn_dir)
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to check configs directory: {}", error),
        })?
    {
        create_dir_all(&byocvpn_dir).await.map_err(|error| {
            ConfigurationError::TunnelConfiguration {
                reason: format!("failed to create configs directory: {}", error),
            }
        })?;
    }

    Ok(byocvpn_dir)
}

fn get_wireguard_config_file_name(
    provider_name: &CloudProviderName,
    region: &str,
    instance_id: &str,
) -> String {
    let safe_id = instance_id.replace('/', "_");
    format!("{provider_name}-{region}-{safe_id}.conf")
}
