mod network_security_group;
mod resource_group;
mod subnet;
mod virtual_network;

use byocvpn_core::error::Result;
use log::info;
pub use network_security_group::ensure_network_security_group;
pub use resource_group::ensure_resource_group;
pub use subnet::ensure_subnet;
pub use virtual_network::ensure_virtual_network;

use crate::{account::ensure_providers_registered, client::AzureClient};

pub struct RegionNetwork {
    pub subnet_id: String,
    pub network_security_group_id: String,
}

pub async fn ensure_region_network(client: &AzureClient, location: &str) -> Result<RegionNetwork> {
    ensure_providers_registered(client).await?;
    ensure_resource_group(client, location).await?;
    let region_network = ensure_virtual_network_resources(client, location).await?;
    info!("[Azure] Region '{location}' network ready.");
    Ok(region_network)
}

pub async fn ensure_virtual_network_resources(
    client: &AzureClient,
    location: &str,
) -> Result<RegionNetwork> {
    let network_security_group_id = ensure_network_security_group(client, location).await?;
    ensure_virtual_network(client, location).await?;
    let subnet_id = ensure_subnet(client, location, &network_security_group_id).await?;
    Ok(RegionNetwork {
        subnet_id,
        network_security_group_id,
    })
}
