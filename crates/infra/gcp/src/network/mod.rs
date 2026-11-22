pub mod firewall;
mod subnet;
pub mod vpc;

pub use firewall::ensure_firewall_rules;
pub use subnet::ensure_subnet;
use tracing::info;
pub use vpc::ensure_vpc;

use crate::{client::GcpClient, error::Result};

pub struct RegionNetwork {
    pub subnet_self_link: String,
}

pub async fn ensure_region_network(client: &GcpClient, region: &str) -> Result<RegionNetwork> {
    let subnet_self_link = ensure_subnet(client, region).await?;
    info!(%region, "region network ready");
    Ok(RegionNetwork { subnet_self_link })
}
