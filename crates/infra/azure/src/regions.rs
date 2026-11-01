use byocvpn_core::error::Result;

use crate::client::AzureClient;
use crate::constants::REGION_CONTINENTS;
use crate::models::LocationListResponse;

const API_VERSION_LOCATIONS: &str = "2022-12-01";

pub async fn list_regions(client: &AzureClient) -> Result<Vec<(String, String)>> {
    let path = client.build_subscription_path("/locations");
    let url = AzureClient::build_arm_url(&path, API_VERSION_LOCATIONS);

    let response: LocationListResponse = client
        .get(&url)
        .await
        .map_err(|error| error.with_context("listing regions"))?;

    let regions = response
        .value
        .unwrap_or_default()
        .into_iter()
        .filter_map(|location_item| {
            let name = location_item.name?;
            let category = location_item
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.region_category.as_deref())?;
            if category != "Recommended" {
                return None;
            }
            let continent = REGION_CONTINENTS
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix))
                .map_or_else(
                    || "Other".to_string(),
                    |(_, continent)| continent.to_string(),
                );
            Some((name, continent))
        })
        .collect();

    Ok(regions)
}
