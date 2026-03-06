/// Persistent ledger of VPN session resource usage for cost estimation.
///
/// Each entry tracks one VPN session (one spawned instance) from creation
/// through termination, including bytes transferred through the tunnel.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single VPN session record stored in the Tauri store under
/// key `"ledger/{instance_id}"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    /// Cloud provider instance ID (e.g. AWS instance ID, Azure `{rg}/{vm}` etc.)
    pub instance_id: String,
    /// Cloud provider name: "aws", "azure", "gcp", "oracle"
    pub provider: String,
    /// Region the instance was spawned in.
    pub region: String,
    /// Instance type / VM size used (e.g. "t2.micro", "Standard_B1s", "e2-micro").
    pub instance_type: String,
    /// When the instance was successfully created.
    pub launched_at: DateTime<Utc>,
    /// When the instance was terminated. `None` while still running.
    pub terminated_at: Option<DateTime<Utc>>,
    /// Cumulative bytes sent through the WireGuard tunnel. Updated every ~60s
    /// while the UI is open and connected.
    pub bytes_sent: u64,
    /// Cumulative bytes received through the WireGuard tunnel.
    pub bytes_received: u64,
}

impl LedgerEntry {
    /// Tauri store key for this entry.
    pub fn store_key(instance_id: &str) -> String {
        format!("ledger/{}", instance_id)
    }
}
