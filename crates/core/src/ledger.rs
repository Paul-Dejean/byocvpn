use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::cloud_provider::CloudProviderName;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    pub instance_id: String,
    pub provider: CloudProviderName,
    pub region: String,
    pub instance_type: String,
    pub launched_at: DateTime<Utc>,
    pub terminated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub idle_since: Option<DateTime<Utc>>,
    #[serde(default)]
    pub setup_complete: bool,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_sent: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_received: u64,
}

impl LedgerEntry {
    pub fn build_store_key(instance_id: &str) -> String {
        format!("ledger/{}", instance_id)
    }

    pub fn mark_terminated(&mut self) {
        self.terminated_at = Some(Utc::now());
    }

    pub fn mark_setup_complete(&mut self) {
        self.setup_complete = true;
    }

    pub fn mark_connected(&mut self) {
        self.idle_since = None;
    }

    pub fn mark_idle_since(&mut self, idle_since: DateTime<Utc>) {
        self.idle_since = Some(idle_since);
    }

    pub fn update_metrics(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.bytes_sent = bytes_sent;
        self.bytes_received = bytes_received;
    }
}
