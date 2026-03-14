use byocvpn_core::{credentials::CredentialStore, error::Result};
use serde::{Deserialize, Serialize};

use crate::provider::AzureProviderConfig;

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
            subscription_id: store.require("AZURE", "subscription_id")?,
            tenant_id: store.require("AZURE", "tenant_id")?,
            client_id: store.require("AZURE", "client_id")?,
            client_secret: store.require("AZURE", "client_secret")?,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        store.set("AZURE", "subscription_id", &self.subscription_id);
        store.set("AZURE", "tenant_id", &self.tenant_id);
        store.set("AZURE", "client_id", &self.client_id);
        store.set("AZURE", "client_secret", &self.client_secret);
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
