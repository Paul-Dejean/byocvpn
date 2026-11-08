use std::path::PathBuf;

use ini::{Error as IniError, Ini};
use log::debug;
use tokio::fs::{create_dir_all, try_exists};

use crate::{
    config::get_data_directory,
    error::{CredentialsError, Result},
};

async fn get_credentials_path() -> Result<PathBuf> {
    let dir = get_data_directory()?;
    if !try_exists(&dir)
        .await
        .map_err(|source| CredentialsError::Directory {
            operation: "check",
            path: dir.clone(),
            source,
        })?
    {
        debug!("Creating credentials directory: {}", dir.display());
        create_dir_all(&dir)
            .await
            .map_err(|source| CredentialsError::Directory {
                operation: "create",
                path: dir.clone(),
                source,
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
        let path = get_credentials_path().await?;
        debug!("Loading credentials from: {}", path.display());
        let ini = if path.exists() {
            let loaded = Ini::load_from_file(&path).map_err(|error| match error {
                IniError::Io(io_error) => CredentialsError::FileRead(io_error),
                IniError::Parse(parse_error) => CredentialsError::InvalidFormat(parse_error),
            })?;
            debug!("Credentials file loaded successfully");
            loaded
        } else {
            debug!("No credentials file found, starting with empty store");
            Ini::new()
        };
        Ok(Self { ini, path })
    }

    pub fn save(&self) -> Result<()> {
        debug!("Saving credentials to: {}", self.path.display());
        self.ini
            .write_to_file(&self.path)
            .map_err(CredentialsError::FileSave)?;
        debug!("Credentials saved successfully");
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
            .map(std::string::ToString::to_string)
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

pub trait StoredCredentials: Sized {
    const CREDENTIALS_SECTION: &'static str;

    fn from_store(store: &CredentialStore) -> Result<Self>;
    fn write_to_store(&self, store: &mut CredentialStore);
}
