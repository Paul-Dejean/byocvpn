use byocvpn_core::{credentials::CredentialStore, error::Result};
use serde::{Deserialize, Serialize};

use crate::provider::AzureProviderConfig;

const CREDENTIALS_SECTION: &str = "AZURE";
const SUBSCRIPTION_ID_FIELD: &str = "subscription_id";
const TENANT_ID_FIELD: &str = "tenant_id";
const CLIENT_ID_FIELD: &str = "client_id";
const CLIENT_SECRET_FIELD: &str = "client_secret";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCredentials {
    pub subscription_id: String,
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
}

impl AzureCredentials {
    pub fn from_store(store: &CredentialStore) -> Result<Self> {
        Ok(Self {
            subscription_id: store.require(CREDENTIALS_SECTION, SUBSCRIPTION_ID_FIELD)?,
            tenant_id: store.require(CREDENTIALS_SECTION, TENANT_ID_FIELD)?,
            client_id: store.require(CREDENTIALS_SECTION, CLIENT_ID_FIELD)?,
            client_secret: store.require(CREDENTIALS_SECTION, CLIENT_SECRET_FIELD)?,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        store.set(CREDENTIALS_SECTION, SUBSCRIPTION_ID_FIELD, &self.subscription_id);
        store.set(CREDENTIALS_SECTION, TENANT_ID_FIELD, &self.tenant_id);
        store.set(CREDENTIALS_SECTION, CLIENT_ID_FIELD, &self.client_id);
        store.set(CREDENTIALS_SECTION, CLIENT_SECRET_FIELD, &self.client_secret);
    }
}

impl From<AzureCredentials> for AzureProviderConfig {
    fn from(credentials: AzureCredentials) -> Self {
        Self {
            subscription_id: credentials.subscription_id,
            tenant_id: credentials.tenant_id,
            client_id: credentials.client_id,
            client_secret: credentials.client_secret,
        }
    }
}
