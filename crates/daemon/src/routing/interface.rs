use byocvpn_core::error::{ConfigurationError, Result};
use net_route::Handle;

pub async fn get_ifindex(interface: &str) -> Result<u32> {
    let handle = Handle::new()?;

    if interface == "default" {
        // Get the default route
        let default_route =
            handle
                .default_route()
                .await?
                .ok_or(ConfigurationError::RouteConfiguration {
                    reason: "No default route found".to_string(),
                })?;
        default_route.ifindex.ok_or(
            ConfigurationError::RouteConfiguration {
                reason: "Default route has no interface index".to_string(),
            }
            .into(),
        )
    } else {
        net_route::ifname_to_index(interface).ok_or(
            ConfigurationError::RouteConfiguration {
                reason: format!("Failed to get interface index for {}", interface),
            }
            .into(),
        )
    }
}
