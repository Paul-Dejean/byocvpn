use std::path::PathBuf;

use ini::Ini;
use tokio::fs::{create_dir_all, try_exists};

use crate::error::{ConfigurationError, CredentialsError, Result};

async fn credentials_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(ConfigurationError::HomeDirectoryNotAvailable)?;
    let dir = home_dir.join(".byocvpn");
    if !try_exists(&dir)
        .await
        .map_err(|error| CredentialsError::FileReadFailed {
            reason: format!("failed to check credentials directory: {}", error),
        })?
    {
        create_dir_all(&dir)
            .await
            .map_err(|error| CredentialsError::FileSaveFailed {
                reason: format!("failed to create credentials directory: {}", error),
            })?;
    }
    Ok(dir.join("credentials"))
}

pub struct CredentialStore {
    ini: Ini,
    path: PathBuf,
}

impl CredentialStore {
    pub async fn load() -> Result<Self> {
        let path = credentials_path().await?;
        let ini = if path.exists() {
            Ini::load_from_file(&path).map_err(|error| match error {
                ini::Error::Io(io_error) => CredentialsError::FileReadFailed {
                    reason: io_error.to_string(),
                },
                ini::Error::Parse(parse_error) => CredentialsError::InvalidFormat {
                    reason: parse_error.to_string(),
                },
            })?
        } else {
            Ini::new()
        };
        Ok(Self { ini, path })
    }

    pub fn save(&self) -> Result<()> {
        self.ini
            .write_to_file(&self.path)
            .map_err(|error| CredentialsError::FileSaveFailed {
                reason: error.to_string(),
            })?;
        Ok(())
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.ini.section(Some(section))?.get(key)
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.ini.with_section(Some(section)).set(key, value);
    }

    pub fn require(&self, section: &str, key: &str) -> Result<String> {
        self.get(section, key)
            .map(|value| value.to_string())
            .ok_or_else(|| {
                CredentialsError::MissingField {
                    field: key.to_string(),
                }
                .into()
            })
    }

    pub fn delete_section(&mut self, section: &str) {
        self.ini.delete(Some(section));
    }
}
