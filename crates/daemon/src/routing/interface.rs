#[cfg(target_os = "linux")]
use std::ffi::CString;

use byocvpn_core::error::{ConfigurationError, Result};
use net_route::Handle;

#[cfg(target_os = "macos")]
fn resolve_interface_index(interface: &str) -> Option<u32> {
    net_route::ifname_to_index(interface)
}

#[cfg(target_os = "linux")]
fn resolve_interface_index(interface: &str) -> Option<u32> {
    let cstr = CString::new(interface).ok()?;
    let index = unsafe { libc::if_nametoindex(cstr.as_ptr()) };
    if index == 0 { None } else { Some(index) }
}

#[cfg(windows)]
fn resolve_interface_index(interface: &str) -> Option<u32> {
    use std::ptr;
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
        GAA_FLAG_SKIP_MULTICAST, GET_ADAPTERS_ADDRESSES_FLAGS,
    };

    let flags = GET_ADAPTERS_ADDRESSES_FLAGS(
        GAA_FLAG_SKIP_ANYCAST.0 | GAA_FLAG_SKIP_MULTICAST.0 | GAA_FLAG_SKIP_DNS_SERVER.0,
    );
    let mut buf_len: u32 = 15000;
    let mut buffer: Vec<u8> = vec![0u8; buf_len as usize];

    let ret = unsafe {
        GetAdaptersAddresses(
            0, // AF_UNSPEC
            flags,
            Some(ptr::null_mut()),
            Some(buffer.as_mut_ptr() as *mut _),
            &mut buf_len,
        )
    };

    if ret != 0 {
        log::error!("[resolve_interface_index] GetAdaptersAddresses failed with error code {}", ret);
        return None;
    }

    let mut adapter = buffer.as_ptr() as *const windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;
    while !adapter.is_null() {
        let a = unsafe { &*adapter };
        if let Ok(friendly_name) = unsafe { a.FriendlyName.to_string() } {
            if friendly_name == interface {
                let ifindex = unsafe { a.Anonymous1.Anonymous.IfIndex };
                return if ifindex != 0 { Some(ifindex) } else { Some(a.Ipv6IfIndex) };
            }
        }
        adapter = a.Next;
    }

    log::error!("[resolve_interface_index] no adapter matched: {}", interface);
    None
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
        // On Windows, WinTun adapters may not be immediately visible to the
        // network stack after creation. Retry a few times to allow the NDIS
        // registration to complete.
        let max_attempts = if cfg!(windows) { 10 } else { 1 };
        for attempt in 1..=max_attempts {
            if let Some(index) = resolve_interface_index(interface) {
                return Ok(index);
            }
            if attempt < max_attempts {
                log::info!(
                    "Waiting for interface {} to appear (attempt {}/{})",
                    interface, attempt, max_attempts
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
        Err(ConfigurationError::RouteConfiguration {
            reason: format!("Failed to get interface index for {}", interface),
        }
        .into())
    }
}
