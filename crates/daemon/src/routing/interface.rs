use std::ffi::CString;

use byocvpn_core::error::{ConfigurationError, Result};
use net_route::Handle;

#[cfg(target_os = "linux")]
fn resolve_interface_index(interface: &str) -> Option<u32> {
    let cstr = CString::new(interface).ok()?;
    let index = unsafe { libc::if_nametoindex(cstr.as_ptr()) };
    if index == 0 { None } else { Some(index) }
}

#[cfg(not(target_os = "linux"))]
fn resolve_interface_index(interface: &str) -> Option<u32> {
    net_route::ifname_to_index(interface)
}

pub async fn get_ifindex(interface: &str) -> Result<u32> {
    let handle = Handle::new().map_err(|error| ConfigurationError::RouteConfiguration {
        reason: format!("failed to create route handle: {}", error),
    })?;

    if interface == "default" {
        let default_route = handle
            .default_route()
            .await
            .map_err(|error| ConfigurationError::RouteConfiguration {
                reason: format!("failed to query default route: {}", error),
            })?
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
        resolve_interface_index(interface).ok_or(
            ConfigurationError::RouteConfiguration {
                reason: format!("Failed to get interface index for {}", interface),
            }
            .into(),
        )
    }
}
