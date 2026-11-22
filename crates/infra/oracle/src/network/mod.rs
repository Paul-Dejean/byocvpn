mod internet_gateway;
mod security_list;
mod subnet;
mod vcn;

pub use internet_gateway::ensure_internet_gateway;
pub use security_list::ensure_security_list;
pub use subnet::ensure_subnet;
use tracing::info;
pub use vcn::ensure_vcn;

use crate::{client::OciClient, error::Result};

pub struct RegionNetwork {
    pub subnet_id: String,
    pub subnet_has_ipv6: bool,
}

pub async fn ensure_region_network(client: &OciClient) -> Result<RegionNetwork> {
    let vcn = ensure_vcn(client).await?;
    ensure_internet_gateway(client, &vcn).await?;
    let security_list_id = ensure_security_list(client, &vcn).await?;
    let subnet_id = ensure_subnet(client, &vcn, &security_list_id).await?;
    info!(region = %client.get_region(), "region network ready");
    Ok(RegionNetwork {
        subnet_id,
        subnet_has_ipv6: vcn.has_ipv6(),
    })
}
