use std::sync::Arc;

use log::*;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

pub struct ProviderStore(Arc<Store<Wry>>);

impl ProviderStore {
    pub fn open(app_handle: &AppHandle) -> Option<Self> {
        app_handle.store("providers.json").ok().map(ProviderStore)
    }

    pub fn mark_provisioned(&self, provider: &str) {
        self.0.set(
            Self::provisioned_key(provider),
            serde_json::Value::Bool(true),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save provider store: {}", error);
        }
    }

    pub fn clear_provisioned(&self, provider: &str) {
        self.0.delete(Self::provisioned_key(provider));
        if let Err(error) = self.0.save() {
            warn!("Failed to save provider store: {}", error);
        }
    }

    pub fn mark_region_enabled(&self, provider: &str, region: &str) {
        self.0.set(
            Self::region_key(provider, region),
            serde_json::Value::Bool(true),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save provider store: {}", error);
        }
    }

    fn provisioned_key(provider: &str) -> String {
        format!("provisioned/{}", provider)
    }

    fn region_key(provider: &str, region: &str) -> String {
        format!("enabled_regions/{}/{}", provider, region)
    }
}
