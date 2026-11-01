use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    cloud_provider::{CloudProviderName, InstanceInfo},
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
    pub fn build_for_launched_instance(instance: &InstanceInfo) -> Self {
        Self {
            instance_id: instance.id.clone(),
            provider: instance.provider.clone(),
            region: instance.region.clone(),
            instance_type: instance.instance_type.clone(),
            launched_at: instance.launched_at.unwrap_or_else(Utc::now),
            terminated_at: None,
            idle_since: Some(Utc::now()),
            setup_complete: false,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    pub fn mark_terminated(&mut self) {
        self.terminated_at = Some(Utc::now());
    }

    pub const fn mark_setup_complete(&mut self) {
        self.setup_complete = true;
    }

    pub const fn mark_connected(&mut self) {
        self.idle_since = None;
    }

    pub const fn mark_idle_since(&mut self, idle_since: DateTime<Utc>) {
        self.idle_since = Some(idle_since);
    }

    pub const fn update_metrics(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.bytes_sent = bytes_sent;
        self.bytes_received = bytes_received;
    }
}

#[derive(Default, Serialize, Deserialize)]
struct LedgerData {
    #[serde(default)]
    entries: BTreeMap<String, LedgerEntry>,
}

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

    pub fn mark_setup_complete(&self, instance_id: &str) {
        self.modify_entry(instance_id, LedgerEntry::mark_setup_complete);
    }

    pub fn mark_connected(&self, instance_id: &str) {
        self.modify_entry(instance_id, LedgerEntry::mark_connected);
    }

    pub fn mark_idle_since(&self, instance_id: &str, idle_since: DateTime<Utc>) {
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
