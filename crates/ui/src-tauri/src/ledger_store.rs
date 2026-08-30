use std::{collections::HashSet, sync::Arc};

use byocvpn_core::{cloud_provider::CloudProviderName, ledger::LedgerEntry};
use chrono::{DateTime, Utc};
use log::*;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

pub struct LedgerStore(Arc<Store<Wry>>);

impl LedgerStore {
    pub fn open(app_handle: &AppHandle) -> Option<Self> {
        app_handle
            .store("byocvpn-ledger.json")
            .ok()
            .map(LedgerStore)
    }

    pub fn set_entry(&self, entry: &LedgerEntry) {
        self.write_entry(entry);
        self.persist();
    }

    pub fn mark_terminated(&self, instance_id: &str) {
        self.modify_entry(instance_id, |entry| entry.mark_terminated());
    }

    pub fn mark_setup_complete(&self, instance_id: &str) {
        self.modify_entry(instance_id, |entry| entry.mark_setup_complete());
    }

    pub fn mark_connected(&self, instance_id: &str) {
        self.modify_entry(instance_id, |entry| entry.mark_connected());
    }

    pub fn mark_idle_since(&self, instance_id: &str, idle_since: DateTime<Utc>) {
        self.modify_entry(instance_id, |entry| entry.mark_idle_since(idle_since));
    }

    pub fn update_metrics(&self, instance_id: &str, bytes_sent: u64, bytes_received: u64) {
        self.modify_entry(instance_id, |entry| {
            entry.update_metrics(bytes_sent, bytes_received)
        });
    }

    pub fn reconcile_terminated(
        &self,
        running_ids: &HashSet<&str>,
        queried_providers: &[CloudProviderName],
    ) {
        for mut entry in self.all_entries() {
            if entry.terminated_at.is_none()
                && queried_providers.contains(&entry.provider)
                && !running_ids.contains(entry.instance_id.as_str())
            {
                entry.mark_terminated();
                self.write_entry(&entry);
            }
        }
        self.persist();
    }

    pub fn get_entry(&self, instance_id: &str) -> Option<LedgerEntry> {
        self.deserialize_entry(instance_id)
    }

    pub fn running_entries(&self) -> Vec<LedgerEntry> {
        self.all_entries()
            .into_iter()
            .filter(|entry| entry.terminated_at.is_none())
            .collect()
    }

    pub fn all_entries(&self) -> Vec<LedgerEntry> {
        self.0
            .keys()
            .into_iter()
            .filter(|key| key.starts_with("ledger/"))
            .filter_map(|key| self.deserialize_entry_by_key(&key))
            .collect()
    }

    fn modify_entry(&self, instance_id: &str, apply_change: impl FnOnce(&mut LedgerEntry)) {
        if let Some(mut entry) = self.deserialize_entry(instance_id) {
            apply_change(&mut entry);
            self.write_entry(&entry);
            self.persist();
        }
    }

    fn write_entry(&self, entry: &LedgerEntry) {
        self.0.set(
            LedgerEntry::build_store_key(&entry.instance_id),
            serde_json::to_value(entry).unwrap_or_else(|error| {
                warn!("Failed to serialize ledger entry: {}", error);
                serde_json::Value::Null
            }),
        );
    }

    fn persist(&self) {
        if let Err(error) = self.0.save() {
            warn!("Failed to save ledger store: {}", error);
        }
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
