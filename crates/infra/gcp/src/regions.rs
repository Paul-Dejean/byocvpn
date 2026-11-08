use byocvpn_core::{cloud_provider::Region, error::Result};
use google_cloud_compute_v1::model::region::Status as RegionStatus;

use crate::{client::GcpClient, error::convert_sdk_error};

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
    let response = client
        .regions
        .list()
        .set_project(&client.project_id)
        .send()
        .await
        .map_err(|error| convert_sdk_error("listing regions", error))?;

    Ok(response
        .items
        .into_iter()
        .filter_map(|region| {
            let name = region.name?;
            (region.status == Some(RegionStatus::Up)).then(|| Region {
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
