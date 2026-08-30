use byocvpn_core::{
    credentials::{CredentialStore, StoredCredentials},
    error::Result,
};
use serde::{Deserialize, Serialize};

use crate::provider::AzureProviderConfig;

const SUBSCRIPTION_ID_FIELD: &str = "subscription_id";
const TENANT_ID_FIELD: &str = "tenant_id";
const APPLICATION_ID_FIELD: &str = "application_id";
const SECRET_VALUE_FIELD: &str = "secret_value";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AzureCredentials {
    pub subscription_id: String,
    pub tenant_id: String,
    pub application_id: String,
    pub secret_value: String,
}

impl StoredCredentials for AzureCredentials {
    const CREDENTIALS_SECTION: &'static str = "AZURE";

    fn from_store(store: &CredentialStore) -> Result<Self> {
        Ok(Self {
            subscription_id: store.require(Self::CREDENTIALS_SECTION, SUBSCRIPTION_ID_FIELD)?,
            tenant_id: store.require(Self::CREDENTIALS_SECTION, TENANT_ID_FIELD)?,
            application_id: store.require(Self::CREDENTIALS_SECTION, APPLICATION_ID_FIELD)?,
            secret_value: store.require(Self::CREDENTIALS_SECTION, SECRET_VALUE_FIELD)?,
        })
    }

    fn write_to_store(&self, store: &mut CredentialStore) {
        store.set(
            Self::CREDENTIALS_SECTION,
            SUBSCRIPTION_ID_FIELD,
            &self.subscription_id,
        );
        store.set(Self::CREDENTIALS_SECTION, TENANT_ID_FIELD, &self.tenant_id);
        store.set(
            Self::CREDENTIALS_SECTION,
            APPLICATION_ID_FIELD,
            &self.application_id,
        );
        store.set(
            Self::CREDENTIALS_SECTION,
            SECRET_VALUE_FIELD,
            &self.secret_value,
        );
    }
}

impl From<AzureCredentials> for AzureProviderConfig {
    fn from(credentials: AzureCredentials) -> Self {
        Self {
            subscription_id: credentials.subscription_id,
            tenant_id: credentials.tenant_id,
            application_id: credentials.application_id,
            secret_value: credentials.secret_value,
        }
    }
}
