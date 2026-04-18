use byocvpn_core::{credentials::CredentialStore, error::Result};
use serde::{Deserialize, Serialize};

use crate::provider::AwsProviderConfig;

const CREDENTIALS_SECTION: &str = "AWS";
const ACCESS_KEY_ID_FIELD: &str = "access_key_id";
const SECRET_ACCESS_KEY_FIELD: &str = "secret_access_key";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl AwsCredentials {
    pub fn from_store(store: &CredentialStore) -> Result<Self> {
        Ok(Self {
            access_key_id: store.require(CREDENTIALS_SECTION, ACCESS_KEY_ID_FIELD)?,
            secret_access_key: store.require(CREDENTIALS_SECTION, SECRET_ACCESS_KEY_FIELD)?,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        store.set(
            CREDENTIALS_SECTION,
            ACCESS_KEY_ID_FIELD,
            &self.access_key_id,
        );
        store.set(
            CREDENTIALS_SECTION,
            SECRET_ACCESS_KEY_FIELD,
            &self.secret_access_key,
        );
    }
}

impl From<AwsCredentials> for AwsProviderConfig {
    fn from(credentials: AwsCredentials) -> Self {
        Self {
            access_key_id: Some(credentials.access_key_id),
            secret_access_key: Some(credentials.secret_access_key),
        }
    }
}
