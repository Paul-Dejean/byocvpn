use byocvpn_core::{credentials::CredentialStore, error::Result};
use serde::{Deserialize, Serialize};

use crate::provider::GcpProviderConfig;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcpCredentials {
    pub project_id: String,
    pub service_account_json: String,
}

impl GcpCredentials {
    pub fn from_store(store: &CredentialStore) -> Result<Self> {
        let project_id = store.require("GCP", "project_id")?;
        let service_account_json = store.require("GCP", "service_account_json")?;
        Ok(Self {
            project_id,
            service_account_json,
        })
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        // Compact the JSON to strip any literal newlines that would break INI parsing
        let compact = serde_json::from_str::<serde_json::Value>(&self.service_account_json)
            .ok()
            .and_then(|v| serde_json::to_string(&v).ok())
            .unwrap_or_else(|| self.service_account_json.clone());
        store.set("GCP", "project_id", &self.project_id);
        store.set("GCP", "service_account_json", &compact);
    }
}

impl From<GcpCredentials> for GcpProviderConfig {
    fn from(credentials: GcpCredentials) -> Self {
        Self {
            service_account_json: credentials.service_account_json,
        }
    }
}
