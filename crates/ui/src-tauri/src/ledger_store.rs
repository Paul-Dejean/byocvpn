use std::{collections::HashSet, sync::Arc};

use byocvpn_core::ledger::LedgerEntry;
use serde_json::Value;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

pub struct LedgerStore(Arc<Store<Wry>>);

impl LedgerStore {
    pub fn open(app_handle: &AppHandle) -> Option<Self> {
        app_handle.store("ledger.json").ok().map(LedgerStore)
    }

    pub fn set_entry(&self, entry: &LedgerEntry) {
        self.0.set(
            LedgerEntry::build_store_key(&entry.instance_id),
            serde_json::to_value(entry).unwrap_or_default(),
        );
        let _ = self.0.save();
    }

    pub fn mark_terminated(&self, instance_id: &str) {
        let key = LedgerEntry::build_store_key(instance_id);
        if let Some(mut entry) = self.deserialize_entry(instance_id) {
            entry.mark_terminated();
            self.0.set(key, serde_json::to_value(&entry).unwrap_or_default());
            let _ = self.0.save();
        }
    }

    pub fn update_metrics(&self, instance_id: &str, bytes_sent: u64, bytes_received: u64) {
        let key = LedgerEntry::build_store_key(instance_id);
        if let Some(mut entry) = self.deserialize_entry(instance_id) {
            entry.update_metrics(bytes_sent, bytes_received);
            self.0.set(key, serde_json::to_value(&entry).unwrap_or_default());
            let _ = self.0.save();
        }
    }

    pub fn reconcile_terminated(&self, running_ids: &HashSet<&str>, queried_providers: &[&str]) {
        let keys: Vec<String> = self
            .0
            .keys()
            .into_iter()
            .filter(|key| key.starts_with("ledger/"))
            .collect();

        for key in keys {
            if let Some(mut entry) = self.deserialize_entry_by_key(&key) {
                if entry.terminated_at.is_none()
                    && queried_providers.contains(&entry.provider.as_str())
                    && !running_ids.contains(entry.instance_id.as_str())
                {
                    entry.mark_terminated();
                    self.0.set(key, serde_json::to_value(&entry).unwrap_or_default());
                }
            }
        }
        let _ = self.0.save();
    }

    pub fn all_entries(&self) -> Vec<Value> {
        self.0
            .keys()
            .into_iter()
            .filter(|key| key.starts_with("ledger/"))
            .filter_map(|key| self.0.get(&key))
            .collect()
    }

    fn deserialize_entry(&self, instance_id: &str) -> Option<LedgerEntry> {
        let key = LedgerEntry::build_store_key(instance_id);
        self.deserialize_entry_by_key(&key)
    }

    fn deserialize_entry_by_key(&self, key: &str) -> Option<LedgerEntry> {
        let value = self.0.get(key)?;
        serde_json::from_value(value).ok()
    }
}
