use byocvpn_core::{
    credentials::{CredentialStore, StoredCredentials},
    error::Result,
};
use serde::{Deserialize, Serialize};

use crate::provider::GcpProviderConfig;

const PROJECT_ID_FIELD: &str = "project_id";
const SERVICE_ACCOUNT_JSON_FIELD: &str = "service_account_json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcpCredentials {
    pub project_id: String,
    pub service_account_json: String,
}

impl StoredCredentials for GcpCredentials {
    const CREDENTIALS_SECTION: &'static str = "GCP";

    fn from_store(store: &CredentialStore) -> Result<Self> {
        let project_id = store.require(Self::CREDENTIALS_SECTION, PROJECT_ID_FIELD)?;
        let service_account_json =
            store.require(Self::CREDENTIALS_SECTION, SERVICE_ACCOUNT_JSON_FIELD)?;
        Ok(Self {
            project_id,
            service_account_json,
        })
    }

    fn write_to_store(&self, store: &mut CredentialStore) {
        // Compact the JSON to strip any literal newlines that would break INI parsing
        let compact = serde_json::from_str::<serde_json::Value>(&self.service_account_json)
            .ok()
            .and_then(|value| serde_json::to_string(&value).ok())
            .unwrap_or_else(|| self.service_account_json.clone());
        store.set(Self::CREDENTIALS_SECTION, PROJECT_ID_FIELD, &self.project_id);
        store.set(Self::CREDENTIALS_SECTION, SERVICE_ACCOUNT_JSON_FIELD, &compact);
    }
}

impl From<GcpCredentials> for GcpProviderConfig {
    fn from(credentials: GcpCredentials) -> Self {
        Self {
            service_account_json: credentials.service_account_json,
        }
    }
}
