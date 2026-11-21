use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, TimeDelta, Utc};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    app_context::AppContext,
    cloud_provider::{CloudProviderName, PricingInfo},
    error::Result,
    json_store::JsonFileStore,
    ledger::LedgerEntry,
};

const PRICING_CACHE_MAX_AGE_HOURS: i64 = 24 * 7;

pub async fn resolve_instance_pricing(
    context: &AppContext,
    provider_name: CloudProviderName,
    region: &str,
    instance_type: &str,
) -> Result<PricingInfo> {
    if let Some(pricing) = context
        .pricing_cache
        .get_fresh(&provider_name, region, instance_type)
    {
        return Ok(pricing);
    }
    let provider = context.providers.create(provider_name).await?;
    let pricing = provider.fetch_pricing(region, instance_type).await?;
    context
        .pricing_cache
        .insert(&provider_name, region, instance_type, &pricing);
    Ok(pricing)
}

pub async fn list_ledger_entries_with_pricing(context: &AppContext) -> Vec<LedgerEntry> {
    let mut entries = context.ledger.all_entries();
    let mut failed_lookups: HashSet<String> = HashSet::new();
    for entry in &mut entries {
        if entry.pricing.is_some() {
            continue;
        }
        let lookup_key = build_cache_key(entry.provider, &entry.region, &entry.instance_type);
        if failed_lookups.contains(&lookup_key) {
            continue;
        }
        match resolve_instance_pricing(context, entry.provider, &entry.region, &entry.instance_type)
            .await
        {
            Ok(pricing) => {
                context.ledger.set_pricing(&entry.instance_id, pricing);
                entry.pricing = Some(pricing);
            }
            Err(error) => {
                warn!(
                    "could not record pricing for ledger entry {}: {error}",
                    entry.instance_id
                );
                failed_lookups.insert(lookup_key);
            }
        }
    }
    entries
}

#[derive(Debug)]
pub struct PricingCache {
    store: JsonFileStore<PricingCacheData>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PricingCacheData {
    #[serde(default)]
    entries: BTreeMap<String, CachedPricing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPricing {
    pricing: PricingInfo,
    fetched_at: DateTime<Utc>,
}

impl PricingCache {
    pub const FILE_NAME: &'static str = "pricing-cache.json";

    pub fn open() -> Result<Self> {
        Ok(Self {
            store: JsonFileStore::open(Self::FILE_NAME)?,
        })
    }

    pub fn get_fresh(
        &self,
        provider: &CloudProviderName,
        region: &str,
        instance_type: &str,
    ) -> Option<PricingInfo> {
        let key = build_cache_key(*provider, region, instance_type);
        let max_age = TimeDelta::hours(PRICING_CACHE_MAX_AGE_HOURS);
        self.store.read(|data| {
            data.entries
                .get(&key)
                .filter(|cached| Utc::now() - cached.fetched_at < max_age)
                .map(|cached| cached.pricing)
        })
    }

    pub fn insert(
        &self,
        provider: &CloudProviderName,
        region: &str,
        instance_type: &str,
        pricing: &PricingInfo,
    ) {
        let key = build_cache_key(*provider, region, instance_type);
        self.store.modify(|data| {
            data.entries.insert(
                key,
                CachedPricing {
                    pricing: *pricing,
                    fetched_at: Utc::now(),
                },
            );
        });
    }
}

fn build_cache_key(provider: CloudProviderName, region: &str, instance_type: &str) -> String {
    format!("{provider}/{region}/{instance_type}")
}
