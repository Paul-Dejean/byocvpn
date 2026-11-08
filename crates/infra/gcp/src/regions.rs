use byocvpn_core::{cloud_provider::Region, error::Result};

use crate::{client::GcpClient, models::RegionListResponse};

const AVAILABLE_REGION_STATUS: &str = "UP";
const UNKNOWN_CONTINENT: &str = "Other";

const REGION_CONTINENTS: &[(&str, &str)] = &[
    ("us-", "North America"),
    ("northamerica-", "North America"),
    ("southamerica-", "South America"),
    ("europe-", "Europe"),
    ("asia-", "Asia Pacific"),
    ("australia-", "Asia Pacific"),
    ("me-", "Middle East"),
    ("africa-", "Africa"),
];

pub async fn list_regions(client: &GcpClient) -> Result<Vec<Region>> {
    let url = client.build_compute_url("regions");
    let response: RegionListResponse = client
        .get(&url)
        .await
        .map_err(|error| error.with_context("listing regions"))?;

    Ok(response
        .items
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| {
            let name = region.name?;
            (region.status.as_deref() == Some(AVAILABLE_REGION_STATUS)).then(|| Region {
                country: select_continent(&name),
                name,
            })
        })
        .collect())
}

fn select_continent(region_name: &str) -> String {
    REGION_CONTINENTS
        .iter()
        .find(|(prefix, _)| region_name.starts_with(prefix))
        .map_or(UNKNOWN_CONTINENT, |(_, continent)| continent)
        .to_string()
}
