use byocvpn_core::error::{ConfigurationError, Result};

pub fn get_interface_index(interface: &str) -> Result<u32> {
    getifaddrs::if_nametoindex(interface)
        .ok()
        .ok_or_else(|| {
            ConfigurationError::RouteConfiguration {
                reason: format!("Failed to get interface index for {}", interface),
            }
            .into()
        })
}
