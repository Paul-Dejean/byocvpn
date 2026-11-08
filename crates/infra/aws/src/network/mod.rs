mod internet_gateway;
mod security_group;
mod subnet;
mod vpc;

use byocvpn_core::error::Result;
pub use internet_gateway::ensure_internet_gateway;
pub use security_group::ensure_security_group;
pub use subnet::ensure_subnet;
pub use vpc::ensure_vpc;

use crate::client::AwsClient;

pub struct RegionNetwork {
    pub subnet_id: String,
    pub security_group_id: String,
}

pub async fn ensure_region_network(client: &AwsClient) -> Result<RegionNetwork> {
    let vpc_id = ensure_vpc(client).await?;
    ensure_internet_gateway(client, &vpc_id).await?;
    let subnet_id = ensure_subnet(client, &vpc_id).await?;
    let security_group_id = ensure_security_group(client, &vpc_id).await?;
    Ok(RegionNetwork {
        subnet_id,
        security_group_id,
    })
}
