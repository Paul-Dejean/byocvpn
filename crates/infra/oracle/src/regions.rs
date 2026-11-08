use byocvpn_core::cloud_provider::Region;

use crate::{
    client::OciClient,
    error::Result,
    models::{RegionResponse, RegionSubscriptionResponse},
};

const UNKNOWN_CONTINENT: &str = "Unknown";

const REGION_CONTINENTS: &[(&str, &str)] = &[
    ("us-", "North America"),
    ("ca-", "North America"),
    ("mx-", "North America"),
    ("sa-", "South America"),
    ("eu-", "Europe"),
    ("uk-", "Europe"),
    ("ap-", "Asia Pacific"),
    ("me-", "Middle East"),
    ("il-", "Middle East"),
    ("af-", "Africa"),
];

pub async fn list_regions(client: &OciClient) -> Result<Vec<Region>> {
    let regions: Vec<RegionResponse> = client.get(&client.build_identity_url("/regions")).await?;
    Ok(regions
        .into_iter()
        .map(|region| build_region(region.name))
        .collect())
}

pub async fn list_subscribed_regions(client: &OciClient) -> Result<Vec<Region>> {
    let url = client.build_identity_url(&format!(
        "/tenancies/{}/regionSubscriptions",
        client.get_compartment_ocid()
    ));
    let subscriptions: Vec<RegionSubscriptionResponse> = client.get(&url).await?;
    Ok(subscriptions
        .into_iter()
        .map(|subscription| build_region(subscription.region_name))
        .collect())
}

fn build_region(name: String) -> Region {
    Region {
        country: select_continent(&name),
        name,
    }
}

fn select_continent(region_name: &str) -> String {
    REGION_CONTINENTS
        .iter()
        .find(|(prefix, _)| region_name.starts_with(prefix))
        .map_or(UNKNOWN_CONTINENT, |(_, continent)| continent)
        .to_string()
}
