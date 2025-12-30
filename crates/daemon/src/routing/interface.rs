use byocvpn_core::error::{Error, Result};
use net_route::Handle;

pub async fn get_ifindex(interface: &str) -> Result<u32> {
    let handle = Handle::new()?;

    if interface == "default" {
        // Get the default route
        let default_route = handle
            .default_route()
            .await?
            .ok_or_else(|| Error::NetworkConfigError("No default route found".to_string()))?;
        default_route.ifindex.ok_or_else(|| {
            Error::NetworkConfigError("Default route has no interface index".to_string())
        })
    } else {
        net_route::ifname_to_index(interface).ok_or_else(|| {
            Error::NetworkConfigError(format!("Failed to get interface index for {}", interface))
        })
    }
}
