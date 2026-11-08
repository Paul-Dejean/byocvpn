use byocvpn_core::cloud_provider::Region;

use crate::{client::AzureClient, error::Result, models::LocationListResponse};

const API_VERSION_LOCATIONS: &str = "2022-12-01";
const RECOMMENDED_REGION_CATEGORY: &str = "Recommended";
const UNKNOWN_CONTINENT: &str = "Other";

const REGION_CONTINENTS: &[(&str, &str)] = &[
    ("eastus", "North America"),
    ("westus", "North America"),
    ("centralus", "North America"),
    ("northcentralus", "North America"),
    ("southcentralus", "North America"),
    ("westcentralus", "North America"),
    ("canadacentral", "North America"),
    ("canadaeast", "North America"),
    ("mexicocentral", "North America"),
    ("eastus2euap", "North America"),
    ("westus2euap", "North America"),
    ("brazilsouth", "South America"),
    ("brazilsoutheast", "South America"),
    ("chilecentral", "South America"),
    ("northeurope", "Europe"),
    ("westeurope", "Europe"),
    ("uksouth", "Europe"),
    ("ukwest", "Europe"),
    ("francecentral", "Europe"),
    ("francesouth", "Europe"),
    ("germanywestcentral", "Europe"),
    ("germanynorth", "Europe"),
    ("norwayeast", "Europe"),
    ("norwaywest", "Europe"),
    ("swedencentral", "Europe"),
    ("switzerlandnorth", "Europe"),
    ("switzerlandwest", "Europe"),
    ("polandcentral", "Europe"),
    ("italynorth", "Europe"),
    ("spaincentral", "Europe"),
    ("austriaeast", "Europe"),
    ("belgiumcentral", "Europe"),
    ("denmarkeast", "Europe"),
    ("eastasia", "Asia Pacific"),
    ("southeastasia", "Asia Pacific"),
    ("australiaeast", "Asia Pacific"),
    ("australiasoutheast", "Asia Pacific"),
    ("australiacentral", "Asia Pacific"),
    ("japaneast", "Asia Pacific"),
    ("japanwest", "Asia Pacific"),
    ("koreacentral", "Asia Pacific"),
    ("koreasouth", "Asia Pacific"),
    ("centralindia", "Asia Pacific"),
    ("southindia", "Asia Pacific"),
    ("westindia", "Asia Pacific"),
    ("jioindiacentral", "Asia Pacific"),
    ("jioindiawest", "Asia Pacific"),
    ("newzealandnorth", "Asia Pacific"),
    ("indonesiacentral", "Asia Pacific"),
    ("malaysiawest", "Asia Pacific"),
    ("uaenorth", "Middle East"),
    ("uaecentral", "Middle East"),
    ("qatarcentral", "Middle East"),
    ("israelcentral", "Middle East"),
    ("southafricanorth", "Africa"),
    ("southafricawest", "Africa"),
];

pub async fn list_regions(client: &AzureClient) -> Result<Vec<Region>> {
    let path = client.build_subscription_path("/locations");
    let url = AzureClient::build_arm_url(&path, API_VERSION_LOCATIONS);
    let response: LocationListResponse = client.get(&url).await?;

    Ok(response
        .value
        .unwrap_or_default()
        .into_iter()
        .filter_map(|location| {
            let name = location.name?;
            let category = location.metadata?.region_category?;
            (category == RECOMMENDED_REGION_CATEGORY).then(|| Region {
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
