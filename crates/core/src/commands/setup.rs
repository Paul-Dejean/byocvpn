use log::debug;
use serde::{Deserialize, Serialize};

use crate::{cloud_provider::CloudProvider, error::Result};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Region {
    pub name: String,
    pub country: String,
}

pub async fn get_regions(provider: &dyn CloudProvider) -> Result<Vec<Region>> {
    debug!("Fetching available regions from provider");
    let regions = provider.get_regions().await?;
    debug!("Fetched {} regions", regions.len());
    Ok(regions)
}
