use std::{
    net::{SocketAddr, TcpStream},
    str::FromStr,
};

use base64::{Engine, engine::general_purpose};
use boringtun::x25519::{PublicKey, StaticSecret};
use ini::Ini;
use rand::rngs::OsRng;
use serde::Serialize;
use tokio::time::Duration;

use crate::cloud_provider::{CloudProvider, CloudProviderName};

pub mod cloud_provider;
pub mod commands;
pub mod daemon_client;
pub mod error;
pub mod tunnel;
use std::path::PathBuf;

use handlebars::Handlebars;
use tokio::fs;

pub fn generate_keypair() -> (String, String) {
    let private_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    let private_b64 = general_purpose::STANDARD.encode(private_key.to_bytes());
    let public_b64 = general_purpose::STANDARD.encode(public_key.as_bytes());

    (private_b64, public_b64)
}

pub fn can_connect_ipv6() -> bool {
    // Google's IPv6 DNS server (UDP/53, but TCP test works fine)
    let addr: SocketAddr = "[2001:4860:4860::8888]:53".parse().unwrap();

    // Try to open a TCP connection with a 2-second timeout
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
}

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
) -> String {
    let template_text: &str = include_str!("templates/client_config.hbs");

    // 2. Build the context (the data injected into the template)
    let context = ClientConfigContext {
        client_private_key: client_private_key.to_string(),
        server_public_key: server_public_key.to_string(),
        server_ip_v4: server_ip_v4.to_string(),
    };

    // 3. Render the template
    let handlebars_registry = Handlebars::new();

    let config = handlebars_registry
        .render_template(template_text, &context)
        .expect("Failed to render client configuration template");
    println!("{}", &config);
    config
}

async fn get_credentials_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let byocvpn_dir = home_dir.join(".byocvpn");

    // Create the directory if it doesn't exist
    if !byocvpn_dir.exists() {
        fs::create_dir_all(&byocvpn_dir)
            .await
            .map_err(|e| format!("Failed to create .byocvpn directory: {}", e))?;
    }

    Ok(byocvpn_dir.join("credentials"))
}

pub async fn get_configs_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let byocvpn_dir = home_dir.join(".byocvpn").join("configs");
    // Create the directory if it doesn't exist
    if !byocvpn_dir.exists() {
        fs::create_dir_all(&byocvpn_dir)
            .await
            .map_err(|e| format!("Failed to create configs directory: {}", e))?;
    }

    Ok(byocvpn_dir)
}

pub async fn save_credentials(
    cloud_provider_name: &CloudProviderName,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<String, String> {
    println!("Saving credentials for {:?}", cloud_provider_name);
    println!("Private Key: {}", server_private_key);
    println!("Public Key: {}", client_public_key);
    let mut config = Ini::new();
    let credentials_path = get_credentials_path().await?;

    config
        .with_general_section()
        .set("cloud_provider_name", cloud_provider_name.to_string());

    let section = Some(cloud_provider_name.to_string());

    config
        .with_section(section)
        .set("access_key", server_private_key)
        .set("secret_access_key", client_public_key);

    config
        .write_to_file(credentials_path)
        .map_err(|e| e.to_string())?;

    Ok("Credentials saved successfully.".to_string())
}

#[derive(Debug)]
pub struct Credentials {
    pub cloud_provider_name: CloudProviderName,
    pub access_key: String,
    pub secret_access_key: String,
}
pub async fn get_credentials() -> Result<Credentials, String> {
    let credentials_path = get_credentials_path().await?;
    let config = Ini::load_from_file(credentials_path).map_err(|e| e.to_string())?;
    // println!("Loaded credentials file: {:?}", config);
    let cloud_provider_name = config
        .general_section()
        .get("cloud_provider_name")
        .ok_or("Cloud provider name not found in credentials file")?;

    let section = config.section(Some(cloud_provider_name.to_string()));
    if let Some(section) = section {
        let access_key = section.get("access_key").ok_or("Access key not found")?;
        let secret_access_key = section
            .get("secret_access_key")
            .ok_or("Secret access key not found")?;
        Ok(Credentials {
            cloud_provider_name: CloudProviderName::from_str(cloud_provider_name)
                .map_err(|_| "Invalid cloud provider name in credentials file".to_string())?,
            access_key: access_key.to_string(),
            secret_access_key: secret_access_key.to_string(),
        })
    } else {
        Err("Credentials for the specified cloud provider not found".to_string())
    }
}

pub async fn verify_permissions(
    cloud_provider: &dyn CloudProvider,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    cloud_provider.verify_permissions().await
}
