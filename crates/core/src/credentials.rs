use std::{path::PathBuf, str::FromStr};

use ini::Ini;
use tokio::fs::{create_dir_all, try_exists};

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, CredentialsError, Error, Result},
};

/// Extract the first valid PEM block from `raw`, normalising line-endings and
/// stripping any surrounding text (e.g. filename labels, comments).
///
/// Returns the trimmed PEM block, or `raw` if no `-----BEGIN` / `-----END`
/// markers can be found.
fn normalize_pem(raw: &str) -> String {
    // Normalise Windows/old-Mac line endings to LF.
    let normalised = raw.replace("\r\n", "\n").replace('\r', "\n");

    // Locate the first -----BEGIN … ----- line.
    let begin_pos = match normalised.find("-----BEGIN ") {
        Some(pos) => pos,
        None => return normalised.trim().to_string(),
    };

    // Locate the last -----END … ----- line.
    let end_prefix = "-----END ";
    let end_start = match normalised.rfind(end_prefix) {
        Some(pos) => pos,
        None => return normalised[begin_pos..].trim().to_string(),
    };

    // Find the end of the -----END … ----- closing line.
    let rest = &normalised[end_start..];
    let end_line_end = rest
        .find('\n')
        .map(|i| end_start + i + 1)
        .unwrap_or(normalised.len());

    normalised[begin_pos..end_line_end].trim_end().to_string()
}

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

#[derive(Debug)]
pub struct OracleCredentials {
    pub tenancy_ocid: String,
    pub user_ocid: String,
    pub fingerprint: String,
    pub private_key_pem: String,
    pub region: String,
}

pub async fn save_oracle_credentials(
    tenancy_ocid: &str,
    user_ocid: &str,
    fingerprint: &str,
    private_key_pem: &str,
    region: &str,
) -> Result<()> {
    let credentials_path = get_credentials_path().await?;

    // Load existing config to preserve other provider credentials, or start fresh.
    let mut config = if credentials_path.exists() {
        Ini::load_from_file(&credentials_path).map_err(|error| match error {
            ini::Error::Io(io_error) => Error::InputOutput(io_error),
            ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
                reason: parse_error.to_string(),
            }
            .into(),
        })?
    } else {
        Ini::new()
    };

    // Normalise the PEM (strip extra content, fix line endings) then escape
    // newlines as literal \n for storage in the INI file.
    let cleaned_pem = normalize_pem(private_key_pem);
    let pem_single_line = cleaned_pem.replace('\n', "\\n");

    config
        .with_section(Some("ORACLE"))
        .set("tenancy_ocid", tenancy_ocid)
        .set("user_ocid", user_ocid)
        .set("fingerprint", fingerprint)
        .set("private_key_pem", &pem_single_line)
        .set("region", region);

    config.write_to_file(credentials_path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// GCP
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct GcpCredentials {
    pub project_id: String,
    /// Full service-account JSON key (stored base64-encoded in the INI file).
    pub service_account_json: String,
}

pub async fn save_gcp_credentials(project_id: &str, service_account_json: &str) -> Result<()> {
    let credentials_path = get_credentials_path().await?;
    let mut config = if credentials_path.exists() {
        Ini::load_from_file(&credentials_path).map_err(|error| match error {
            ini::Error::Io(io_error) => Error::InputOutput(io_error),
            ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
                reason: parse_error.to_string(),
            }
            .into(),
        })?
    } else {
        Ini::new()
    };

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    let json_b64 = BASE64.encode(service_account_json.as_bytes());

    config
        .with_section(Some("GCP"))
        .set("project_id", project_id)
        .set("service_account_json_b64", &json_b64);

    config.write_to_file(credentials_path)?;
    Ok(())
}

pub async fn get_gcp_credentials() -> Result<GcpCredentials> {
    let credentials_path = get_credentials_path().await?;
    let config = Ini::load_from_file(credentials_path).map_err(|error| match error {
        ini::Error::Io(io_error) => Error::InputOutput(io_error),
        ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
            reason: parse_error.to_string(),
        }
        .into(),
    })?;

    let section = config
        .section(Some("GCP"))
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing [GCP] section in credentials file".to_string(),
        })?;

    let project_id = section
        .get("project_id")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing project_id".to_string(),
        })?
        .to_string();

    let json_b64 =
        section
            .get("service_account_json_b64")
            .ok_or(CredentialsError::InvalidFormat {
                reason: "missing service_account_json_b64".to_string(),
            })?;

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    let json_bytes = BASE64
        .decode(json_b64)
        .map_err(|error| CredentialsError::InvalidFormat {
            reason: format!("Invalid base64 in service_account_json_b64: {}", error),
        })?;
    let service_account_json =
        String::from_utf8(json_bytes).map_err(|error| CredentialsError::InvalidFormat {
            reason: format!("service_account_json is not valid UTF-8: {}", error),
        })?;

    Ok(GcpCredentials {
        project_id,
        service_account_json,
    })
}

pub async fn get_oracle_credentials() -> Result<OracleCredentials> {
    let credentials_path = get_credentials_path().await?;
    let config = Ini::load_from_file(credentials_path).map_err(|error| match error {
        ini::Error::Io(io_error) => Error::InputOutput(io_error),
        ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
            reason: parse_error.to_string(),
        }
        .into(),
    })?;

    let section = config
        .section(Some("ORACLE"))
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing [ORACLE] section in credentials file".to_string(),
        })?;

    let tenancy_ocid = section
        .get("tenancy_ocid")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing tenancy_ocid".to_string(),
        })?
        .to_string();
    let user_ocid = section
        .get("user_ocid")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing user_ocid".to_string(),
        })?
        .to_string();
    let fingerprint = section
        .get("fingerprint")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing fingerprint".to_string(),
        })?
        .to_string();
    let pem_escaped = section
        .get("private_key_pem")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing private_key_pem".to_string(),
        })?;
    let private_key_pem = normalize_pem(&pem_escaped.replace("\\n", "\n"));
    let region = section
        .get("region")
        .ok_or(CredentialsError::InvalidFormat {
            reason: "missing region".to_string(),
        })?
        .to_string();

    Ok(OracleCredentials {
        tenancy_ocid,
        user_ocid,
        fingerprint,
        private_key_pem,
        region,
    })
}
