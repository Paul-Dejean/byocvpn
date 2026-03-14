use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    pub instance_id: String,

    pub provider: String,

    pub region: String,

    pub instance_type: String,

    pub launched_at: DateTime<Utc>,

    pub terminated_at: Option<DateTime<Utc>>,

    pub bytes_sent: u64,

    pub bytes_received: u64,
}

impl LedgerEntry {
    pub fn build_store_key(instance_id: &str) -> String {
        format!("ledger/{}", instance_id)
    }

    pub fn mark_terminated(&mut self) {
        self.terminated_at = Some(Utc::now());
    }

    pub fn update_metrics(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.bytes_sent = bytes_sent;
        self.bytes_received = bytes_received;
    }
}
