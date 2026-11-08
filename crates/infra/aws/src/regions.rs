use byocvpn_core::cloud_provider::Region;
use log::info;

use crate::{client::AwsClient, error::Result};

const REGION_CONTINENTS: &[(&str, &str)] = &[
    ("us", "North America"),
    ("ca", "North America"),
    ("mx", "North America"),
    ("sa", "South America"),
    ("eu", "Europe"),
    ("ap", "Asia Pacific"),
    ("me", "Middle East"),
    ("il", "Middle East"),
    ("af", "Africa"),
];
const UNKNOWN_CONTINENT: &str = "Unknown";

pub async fn list_regions(client: &AwsClient) -> Result<Vec<Region>> {
    info!("[AWS] Fetching regions...");
    let response = client.ec2.describe_regions().send().await?;

    let regions: Vec<Region> = response
        .regions()
        .iter()
        .filter_map(|region| region.region_name())
        .map(|name| Region {
            name: name.to_string(),
            country: select_continent(name),
        })
        .collect();
    info!("[AWS] Fetched {} regions", regions.len());
    Ok(regions)
}

fn select_continent(region_name: &str) -> String {
    let prefix = region_name.split('-').next().unwrap_or_default();
    REGION_CONTINENTS
        .iter()
        .find(|(known_prefix, _)| *known_prefix == prefix)
        .map_or(UNKNOWN_CONTINENT, |(_, continent)| continent)
        .to_string()
}
