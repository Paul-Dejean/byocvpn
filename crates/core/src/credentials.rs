use std::{path::PathBuf, str::FromStr};

use ini::Ini;
use tokio::fs::{create_dir_all, try_exists};

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, CredentialsError, Error, Result},
};

async fn get_credentials_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(ConfigurationError::HomeDirectoryNotAvailable)?;
    let byocvpn_dir = home_dir.join(".byocvpn");

    // Create the directory if it doesn't exist
    if !try_exists(&byocvpn_dir).await? {
        create_dir_all(&byocvpn_dir).await?;
    }

    Ok(byocvpn_dir.join("credentials"))
}

pub async fn save_credentials(
    cloud_provider_name: &CloudProviderName,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<()> {
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

    config.write_to_file(credentials_path)?;

    Ok(())
}

#[derive(Debug)]
pub struct Credentials {
    pub cloud_provider_name: CloudProviderName,
    pub access_key: String,
    pub secret_access_key: String,
}
pub async fn get_credentials() -> Result<Credentials> {
    let credentials_path = get_credentials_path().await?;
    let config = Ini::load_from_file(credentials_path).map_err(|error| match error {
        ini::Error::Io(io_error) => Error::InputOutput(io_error),
        ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
            reason: parse_error.to_string(),
        }
        .into(),
    })?;

    let cloud_provider_name = config.general_section().get("cloud_provider_name").ok_or(
        CredentialsError::InvalidFormat {
            reason: "missing cloud provider name in credentials file".to_string(),
        },
    )?;

    let section = config
        .section(Some(cloud_provider_name.to_string()))
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing cloud provider section in credentials file".to_string(),
        })?;

    let access_key = section
        .get("access_key")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing access key in credentials file".to_string(),
        })?;
    let secret_access_key =
        section
            .get("secret_access_key")
            .ok_or(CredentialsError::InvalidFormat {
                reason: "missing secret access key in credentials file".to_string(),
            })?;
    Ok(Credentials {
        cloud_provider_name: CloudProviderName::from_str(cloud_provider_name).map_err(|error| {
            CredentialsError::InvalidFormat {
                reason: format!("Invalid cloud provider name in credentials file: {error}"),
            }
        })?,
        access_key: access_key.to_string(),
        secret_access_key: secret_access_key.to_string(),
    })
}
