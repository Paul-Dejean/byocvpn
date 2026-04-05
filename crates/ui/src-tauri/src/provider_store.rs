use std::sync::Arc;

use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

pub struct ProviderStore(Arc<Store<Wry>>);

impl ProviderStore {
    pub fn open(app_handle: &AppHandle) -> Option<Self> {
        app_handle.store("providers.json").ok().map(ProviderStore)
    }

    pub fn mark_provisioned(&self, provider: &str) {
        self.0
            .set(Self::provisioned_key(provider), serde_json::Value::Bool(true));
        let _ = self.0.save();
    }

    pub fn clear_provisioned(&self, provider: &str) {
        self.0.delete(Self::provisioned_key(provider));
        let _ = self.0.save();
    }

    pub fn mark_region_enabled(&self, provider: &str, region: &str) {
        self.0
            .set(Self::region_key(provider, region), serde_json::Value::Bool(true));
        let _ = self.0.save();
    }

    fn provisioned_key(provider: &str) -> String {
        format!("provisioned/{}", provider)
    }

    fn region_key(provider: &str, region: &str) -> String {
        format!("enabled_regions/{}/{}", provider, region)
    }
}
