use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{cloud_provider::CloudProviderName, error::Result, json_store::JsonFileStore};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct ProviderFlagsData {
    provisioned: BTreeSet<CloudProviderName>,
    enabled_regions: BTreeMap<CloudProviderName, BTreeSet<String>>,
}

pub struct ProviderFlags {
    store: JsonFileStore<ProviderFlagsData>,
}

impl ProviderFlags {
    pub const FILE_NAME: &'static str = "provider-flags.json";

    pub fn open() -> Result<Self> {
        Ok(Self {
            store: JsonFileStore::open(Self::FILE_NAME)?,
        })
    }

    pub fn is_persisted(&self) -> bool {
        self.store.is_persisted()
    }

    pub fn mark_provisioned(&self, provider: &CloudProviderName) {
        self.store.modify(|data| {
            data.provisioned.insert(provider.clone());
        });
    }

    pub fn clear_provisioned(&self, provider: &CloudProviderName) {
        self.store.modify(|data| {
            data.provisioned.remove(provider);
        });
    }

    pub fn is_provisioned(&self, provider: &CloudProviderName) -> bool {
        self.store.read(|data| data.provisioned.contains(provider))
    }

    pub fn mark_region_enabled(&self, provider: &CloudProviderName, region: &str) {
        self.store.modify(|data| {
            data.enabled_regions
                .entry(provider.clone())
                .or_default()
                .insert(region.to_string());
        });
    }

    pub fn is_region_enabled(&self, provider: &CloudProviderName, region: &str) -> bool {
        self.store.read(|data| {
            data.enabled_regions
                .get(provider)
                .is_some_and(|regions| regions.contains(region))
        })
    }

    pub fn list_enabled_regions(&self, provider: &CloudProviderName) -> Vec<String> {
        self.store.read(|data| {
            data.enabled_regions
                .get(provider)
                .map(|regions| regions.iter().cloned().collect())
                .unwrap_or_default()
        })
    }
}
