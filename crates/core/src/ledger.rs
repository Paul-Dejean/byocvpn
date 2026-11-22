use std::collections::{BTreeMap, HashSet};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    cloud_provider::{CloudProviderName, InstanceInfo, PricingInfo},
    error::Result,
    json_store::JsonFileStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    pub instance_id: String,
    pub provider: CloudProviderName,
    pub region: String,
    pub instance_type: String,
    #[cfg_attr(feature = "specta", specta(type = String))]
    pub launched_at: Timestamp,
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub terminated_at: Option<Timestamp>,
    #[serde(default)]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub idle_since: Option<Timestamp>,
    #[serde(default)]
    pub setup_complete: bool,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_sent: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_received: u64,
    #[serde(default)]
    pub pricing: Option<PricingInfo>,
}

impl LedgerEntry {
    pub fn build_for_launched_instance(
        instance: &InstanceInfo,
        pricing: Option<PricingInfo>,
    ) -> Self {
        Self {
            instance_id: instance.id.clone(),
            provider: instance.provider,
            region: instance.region.clone(),
            instance_type: instance.instance_type.clone(),
            launched_at: instance.launched_at.unwrap_or_else(Timestamp::now),
            terminated_at: None,
            idle_since: Some(Timestamp::now()),
            setup_complete: false,
            bytes_sent: 0,
            bytes_received: 0,
            pricing,
        }
    }

    pub fn mark_terminated(&mut self) {
        self.terminated_at = Some(Timestamp::now());
    }

    pub const fn mark_setup_complete(&mut self) {
        self.setup_complete = true;
    }

    pub const fn mark_connected(&mut self) {
        self.idle_since = None;
    }

    pub const fn mark_idle_since(&mut self, idle_since: Timestamp) {
        self.idle_since = Some(idle_since);
    }

    pub const fn update_metrics(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.bytes_sent = bytes_sent;
        self.bytes_received = bytes_received;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LedgerData {
    #[serde(default)]
    entries: BTreeMap<String, LedgerEntry>,
}

#[derive(Debug)]
pub struct Ledger {
    store: JsonFileStore<LedgerData>,
}

impl Ledger {
    pub const FILE_NAME: &'static str = "ledger.json";

    pub fn open() -> Result<Self> {
        Ok(Self {
            store: JsonFileStore::open(Self::FILE_NAME)?,
        })
    }

    pub fn is_persisted(&self) -> bool {
        self.store.is_persisted()
    }

    pub fn insert_entry(&self, entry: LedgerEntry) {
        self.store.modify(|data| {
            data.entries.insert(entry.instance_id.clone(), entry);
        });
    }

    pub fn contains_entry(&self, instance_id: &str) -> bool {
        self.store
            .read(|data| data.entries.contains_key(instance_id))
    }

    pub fn mark_terminated(&self, instance_id: &str) {
        self.modify_entry(instance_id, LedgerEntry::mark_terminated);
    }

    pub fn set_pricing(&self, instance_id: &str, pricing: PricingInfo) {
        self.modify_entry(instance_id, |entry| entry.pricing = Some(pricing));
    }

    pub fn mark_setup_complete(&self, instance_id: &str) {
        self.modify_entry(instance_id, LedgerEntry::mark_setup_complete);
    }

    pub fn mark_connected(&self, instance_id: &str) {
        self.modify_entry(instance_id, LedgerEntry::mark_connected);
    }

    pub fn mark_idle_since(&self, instance_id: &str, idle_since: Timestamp) {
        self.modify_entry(instance_id, |entry| entry.mark_idle_since(idle_since));
    }

    pub fn update_metrics(&self, instance_id: &str, bytes_sent: u64, bytes_received: u64) {
        self.modify_entry(instance_id, |entry| {
            entry.update_metrics(bytes_sent, bytes_received);
        });
    }

    pub fn reconcile_terminated(
        &self,
        running_ids: &HashSet<&str>,
        queried_providers: &[CloudProviderName],
    ) {
        self.store.modify(|data| {
            for entry in data.entries.values_mut() {
                if entry.terminated_at.is_none()
                    && queried_providers.contains(&entry.provider)
                    && !running_ids.contains(entry.instance_id.as_str())
                {
                    entry.mark_terminated();
                }
            }
        });
    }

    pub fn running_entries(&self) -> Vec<LedgerEntry> {
        self.store.read(|data| {
            data.entries
                .values()
                .filter(|entry| entry.terminated_at.is_none())
                .cloned()
                .collect()
        })
    }

    pub fn all_entries(&self) -> Vec<LedgerEntry> {
        self.store
            .read(|data| data.entries.values().cloned().collect())
    }

    fn modify_entry(&self, instance_id: &str, apply_change: impl FnOnce(&mut LedgerEntry)) {
        self.store.modify(|data| {
            if let Some(entry) = data.entries.get_mut(instance_id) {
                apply_change(entry);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error as StdError, result::Result as StdResult};

    use super::*;

    type TestResult = StdResult<(), Box<dyn StdError>>;

    #[test]
    fn reads_entries_written_with_chrono_timestamps() -> TestResult {
        let json = r#"{
            "instanceId": "i-0123456789abcdef0",
            "provider": "AWS",
            "region": "eu-west-3",
            "instanceType": "t3.micro",
            "launchedAt": "2026-11-20T12:34:56.789012345Z",
            "terminatedAt": null,
            "idleSince": "2026-11-20T13:00:00Z",
            "setupComplete": true,
            "bytesSent": 1024,
            "bytesReceived": 2048,
            "pricing": null
        }"#;

        let entry: LedgerEntry = serde_json::from_str(json)?;

        assert_eq!(
            entry.launched_at,
            "2026-11-20T12:34:56.789012345Z".parse::<Timestamp>()?,
            "fractional seconds written by chrono must round-trip"
        );
        assert_eq!(
            entry.idle_since,
            Some("2026-11-20T13:00:00Z".parse::<Timestamp>()?),
            "whole-second timestamps must round-trip"
        );
        assert!(entry.terminated_at.is_none(), "null stays None");
        Ok(())
    }

    #[test]
    fn writes_timestamps_as_rfc3339_strings() -> TestResult {
        let entry = LedgerEntry {
            instance_id: "i-1".to_string(),
            provider: CloudProviderName::Aws,
            region: "eu-west-3".to_string(),
            instance_type: "t3.micro".to_string(),
            launched_at: "2026-11-20T12:34:56Z".parse()?,
            terminated_at: None,
            idle_since: None,
            setup_complete: false,
            bytes_sent: 0,
            bytes_received: 0,
            pricing: None,
        };

        let json = serde_json::to_string(&entry)?;

        assert!(
            json.contains(r#""launchedAt":"2026-11-20T12:34:56Z""#),
            "launched_at must serialize as an RFC 3339 string, got {json}"
        );
        Ok(())
    }
}
