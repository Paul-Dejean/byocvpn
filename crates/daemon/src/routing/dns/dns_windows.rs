use std::collections::HashMap;
use std::ffi::CStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

use windows_sys::Win32::Foundation::FreeLibrary;
use windows_sys::Win32::NetworkManagement::IpHelper::*;
use windows_sys::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows_sys::Win32::System::LibraryLoader::*;
use windows_sys::Win32::System::Registry::*;

const DNS_REG_KEY_V4: &str = r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
const DNS_REG_KEY_V6: &str = r"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters\Interfaces";

#[derive(Debug)]
struct OriginalDnsConfig {
    ipv4_nameserver: String,
    ipv6_nameserver: String,
}

#[derive(Debug)]
pub struct DnsOverrideGuard {
    /// Map from adapter GUID to (friendly_name, original_config)
    previous: HashMap<String, (String, OriginalDnsConfig)>,
    dns_override_active: bool,
}

impl DnsOverrideGuard {
    pub fn override_system_dns(new_dns_servers: &[&str]) -> Result<Self> {
        if new_dns_servers.is_empty() {
            return Err(ConfigurationError::DnsConfiguration {
                reason: "desired DNS servers list is empty".to_string(),
            }
            .into());
        }

        let adapters = list_connected_adapters().map_err(|e| {
            ConfigurationError::DnsConfiguration {
                reason: format!("failed to list network adapters: {}", e),
            }
        })?;

        info!(
            "Storing previous DNS configuration for {} interfaces",
            adapters.len()
        );

        let mut previous = HashMap::new();
        for adapter in &adapters {
            let config = read_dns_config(&adapter.adapter_guid).map_err(|e| {
                ConfigurationError::DnsConfiguration {
                    reason: format!(
                        "failed to read DNS for {} ({}): {}",
                        adapter.friendly_name, adapter.adapter_guid, e
                    ),
                }
            })?;
            debug!(
                "Original DNS for {} ({}): {:?}",
                adapter.friendly_name, adapter.adapter_guid, config
            );
            previous.insert(
                adapter.adapter_guid.clone(),
                (adapter.friendly_name.clone(), config),
            );
        }

        let ipv4_dns: Vec<&str> = new_dns_servers
            .iter()
            .copied()
            .filter(|s| !s.contains(':'))
            .collect();
        let ipv6_dns: Vec<&str> = new_dns_servers
            .iter()
            .copied()
            .filter(|s| s.contains(':'))
            .collect();

        info!("Setting DNS servers: IPv4={:?}, IPv6={:?}", ipv4_dns, ipv6_dns);

        for adapter in &adapters {
            write_dns_config(&adapter.adapter_guid, &ipv4_dns, &ipv6_dns).map_err(|e| {
                ConfigurationError::DnsConfiguration {
                    reason: format!(
                        "failed to set DNS for {} ({}): {}",
                        adapter.friendly_name, adapter.adapter_guid, e
                    ),
                }
            })?;
            notify_adapter_config_change(&adapter.adapter_guid);
        }

        flush_dns_resolver_cache();

        Ok(Self {
            previous,
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> io::Result<()> {
        if !self.dns_override_active {
            return Ok(());
        }

        info!("Restoring original DNS settings...");
        for (guid, (name, config)) in &self.previous {
            info!("Restoring DNS for {} ({})", name, guid);
            restore_dns_config(guid, config)?;
            notify_adapter_config_change(guid);
        }

        flush_dns_resolver_cache();
        self.dns_override_active = false;
        info!("DNS restoration completed");
        Ok(())
    }
}

impl Drop for DnsOverrideGuard {
    fn drop(&mut self) {
        if let Err(error) = self.restore_previous_dns_configuration() {
            warn!("Failed to restore DNS on drop: {}", error);
        }
    }
}

// --- Adapter enumeration via GetAdaptersAddresses ---

struct AdapterInfo {
    friendly_name: String,
    adapter_guid: String,
}

fn list_connected_adapters() -> io::Result<Vec<AdapterInfo>> {
    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

    let mut size = 0u32;
    let result = unsafe {
        GetAdaptersAddresses(0, flags, ptr::null(), ptr::null_mut(), &mut size)
    };

    if result != windows_sys::Win32::Foundation::ERROR_BUFFER_OVERFLOW {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    let mut buffer = vec![0u8; size as usize];
    let addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    let result = unsafe {
        GetAdaptersAddresses(0, flags, ptr::null(), addresses, &mut size)
    };

    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    let mut adapters = Vec::new();
    let mut current = addresses;

    while !current.is_null() {
        let addr = unsafe { &*current };

        if addr.OperStatus == IfOperStatusUp {
            let friendly_name = unsafe { wide_ptr_to_string(addr.FriendlyName) };
            let adapter_guid = unsafe { ansi_ptr_to_string(addr.AdapterName) };

            if !adapter_guid.is_empty() {
                adapters.push(AdapterInfo {
                    friendly_name,
                    adapter_guid,
                });
            }
        }

        current = addr.Next;
    }

    Ok(adapters)
}

// --- DNS config via registry ---

fn read_dns_config(adapter_guid: &str) -> io::Result<OriginalDnsConfig> {
    let ipv4 = read_registry_nameserver(DNS_REG_KEY_V4, adapter_guid)?;
    let ipv6 = read_registry_nameserver(DNS_REG_KEY_V6, adapter_guid)?;
    Ok(OriginalDnsConfig {
        ipv4_nameserver: ipv4,
        ipv6_nameserver: ipv6,
    })
}

fn write_dns_config(adapter_guid: &str, ipv4: &[&str], ipv6: &[&str]) -> io::Result<()> {
    write_registry_nameserver(DNS_REG_KEY_V4, adapter_guid, &ipv4.join(","))?;
    write_registry_nameserver(DNS_REG_KEY_V6, adapter_guid, &ipv6.join(","))?;
    Ok(())
}

fn restore_dns_config(adapter_guid: &str, config: &OriginalDnsConfig) -> io::Result<()> {
    write_registry_nameserver(DNS_REG_KEY_V4, adapter_guid, &config.ipv4_nameserver)?;
    write_registry_nameserver(DNS_REG_KEY_V6, adapter_guid, &config.ipv6_nameserver)?;
    Ok(())
}

fn read_registry_nameserver(base_key: &str, adapter_guid: &str) -> io::Result<String> {
    let key_path = format!("{}\\{}", base_key, adapter_guid);
    let key_path_w = to_wide(&key_path);

    let mut hkey = ptr::null_mut();
    let result = unsafe {
        RegOpenKeyExW(HKEY_LOCAL_MACHINE, key_path_w.as_ptr(), 0, KEY_READ, &mut hkey)
    };

    if result != 0 {
        return Ok(String::new());
    }

    let value_name = to_wide("NameServer");
    let mut data_size = 0u32;
    let mut data_type = 0u32;

    let result = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            ptr::null(),
            &mut data_type,
            ptr::null_mut(),
            &mut data_size,
        )
    };

    if result != 0 || data_size == 0 {
        unsafe { RegCloseKey(hkey) };
        return Ok(String::new());
    }

    let mut data = vec![0u16; (data_size as usize) / 2];
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            ptr::null(),
            &mut data_type,
            data.as_mut_ptr() as *mut u8,
            &mut data_size,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if result != 0 {
        return Ok(String::new());
    }

    while data.last() == Some(&0) {
        data.pop();
    }

    Ok(String::from_utf16_lossy(&data))
}

fn write_registry_nameserver(base_key: &str, adapter_guid: &str, value: &str) -> io::Result<()> {
    let key_path = format!("{}\\{}", base_key, adapter_guid);
    let key_path_w = to_wide(&key_path);

    let mut hkey = ptr::null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            key_path_w.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if result != 0 {
        return Ok(());
    }

    let value_name = to_wide("NameServer");
    let value_w = to_wide(value);
    let data_size = (value_w.len() * 2) as u32;

    let result = unsafe {
        RegSetValueExW(
            hkey,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr() as *const u8,
            data_size,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    Ok(())
}

// --- System notifications ---

fn notify_adapter_config_change(adapter_guid: &str) {
    type DhcpNotifyFn =
        unsafe extern "system" fn(*const u16, *const u16, i32, u32, u32, u32, i32) -> u32;

    unsafe {
        let lib = LoadLibraryW(to_wide("dhcpcsvc.dll").as_ptr());
        if lib.is_null() {
            debug!("Failed to load dhcpcsvc.dll");
            return;
        }

        if let Some(func) = GetProcAddress(lib, b"DhcpNotifyConfigChange\0".as_ptr()) {
            let notify: DhcpNotifyFn = std::mem::transmute(func);
            let guid_w = to_wide(adapter_guid);
            notify(ptr::null(), guid_w.as_ptr(), 0, 0, 0, 0, 0);
        }

        FreeLibrary(lib);
    }
}

fn flush_dns_resolver_cache() {
    type DnsFlushFn = unsafe extern "system" fn() -> i32;

    unsafe {
        let lib = LoadLibraryW(to_wide("dnsapi.dll").as_ptr());
        if lib.is_null() {
            debug!("Failed to load dnsapi.dll");
            return;
        }

        if let Some(func) = GetProcAddress(lib, b"DnsFlushResolverCache\0".as_ptr()) {
            let flush: DnsFlushFn = std::mem::transmute(func);
            flush();
            debug!("DNS resolver cache flushed");
        }

        FreeLibrary(lib);
    }
}

// --- String helpers ---

fn to_wide(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| unsafe { *ptr.add(i) != 0 }).count();
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

unsafe fn ansi_ptr_to_string(ptr: *const u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr as *const std::ffi::c_char) }
        .to_string_lossy()
        .into_owned()
}
