use byocvpn_core::{
    credentials::CredentialStore,
    error::Result,
};
use serde::{Deserialize, Serialize};

use crate::provider::AwsProviderConfig;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl AwsCredentials {
    pub fn from_store(store: &CredentialStore) -> Result<Self> {
        Ok(Self {
            access_key_id: store.require("AWS", "access_key_id")?,
            secret_access_key: store.require("AWS", "secret_access_key")?,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        store.set("AWS", "access_key_id", &self.access_key_id);
        store.set("AWS", "secret_access_key", &self.secret_access_key);
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
