use std::io;
use std::ptr;

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

use windows_sys::Win32::Foundation::FreeLibrary;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyExW, RegSetValueExW, HKEY_LOCAL_MACHINE,
    KEY_SET_VALUE, REG_DWORD, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE, REG_SZ,
};

const NRPT_POLICY_KEY: &str =
    r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig";
const NRPT_RULE_SUBKEY: &str = "{ByocVPN-DNS-Override}";
const NRPT_VERSION: u32 = 2;
const NRPT_CONFIG_DNS_SERVERS: u32 = 0x08;

#[derive(Debug)]
pub struct DnsOverrideGuard {
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

        add_nrpt_rule(new_dns_servers).map_err(|error| ConfigurationError::DnsConfiguration {
            reason: format!("failed to add NRPT rule: {}", error),
        })?;

        flush_dns_resolver_cache();
        info!("NRPT DNS rule applied for servers: {:?}", new_dns_servers);

        Ok(Self {
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> io::Result<()> {
        if !self.dns_override_active {
            return Ok(());
        }

        remove_nrpt_rule()?;
        flush_dns_resolver_cache();
        self.dns_override_active = false;
        info!("NRPT DNS rule removed");
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

fn add_nrpt_rule(dns_servers: &[&str]) -> io::Result<()> {
    let key_path = format!("{}\\{}", NRPT_POLICY_KEY, NRPT_RULE_SUBKEY);
    let key_path_w = to_wide(&key_path);

    let mut hkey = ptr::null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            key_path_w.as_ptr(),
            0,
            ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            ptr::null::<SECURITY_ATTRIBUTES>(),
            &mut hkey,
            ptr::null_mut(),
        )
    };

    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    let write_result = (|| {
        set_registry_dword(hkey, "Version", NRPT_VERSION)?;
        set_registry_dword(hkey, "ConfigOptions", NRPT_CONFIG_DNS_SERVERS)?;
        set_registry_multi_sz(hkey, "Name", &["."])?;
        set_registry_sz(hkey, "GenericDNSServers", &dns_servers.join(";"))?;
        Ok::<_, io::Error>(())
    })();

    unsafe { RegCloseKey(hkey) };
    write_result
}

fn remove_nrpt_rule() -> io::Result<()> {
    let key_path = format!("{}\\{}", NRPT_POLICY_KEY, NRPT_RULE_SUBKEY);
    let key_path_w = to_wide(&key_path);

    let result = unsafe { RegDeleteKeyExW(HKEY_LOCAL_MACHINE, key_path_w.as_ptr(), 0, 0) };

    const ERROR_FILE_NOT_FOUND: u32 = 2;
    if result != 0 && result != ERROR_FILE_NOT_FOUND {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

type RegistryKey = windows_sys::Win32::System::Registry::HKEY;

fn set_registry_dword(hkey: RegistryKey, name: &str, value: u32) -> io::Result<()> {
    let name_w = to_wide(name);
    let result = unsafe {
        RegSetValueExW(
            hkey,
            name_w.as_ptr(),
            0,
            REG_DWORD,
            &value as *const u32 as *const u8,
            4,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

fn set_registry_sz(hkey: RegistryKey, name: &str, value: &str) -> io::Result<()> {
    let name_w = to_wide(name);
    let value_w = to_wide(value);
    let result = unsafe {
        RegSetValueExW(
            hkey,
            name_w.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr() as *const u8,
            (value_w.len() * 2) as u32,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

fn set_registry_multi_sz(hkey: RegistryKey, name: &str, values: &[&str]) -> io::Result<()> {
    let name_w = to_wide(name);
    let mut data: Vec<u16> = Vec::new();
    for value in values {
        data.extend(value.encode_utf16());
        data.push(0);
    }
    data.push(0);

    let result = unsafe {
        RegSetValueExW(
            hkey,
            name_w.as_ptr(),
            0,
            REG_MULTI_SZ,
            data.as_ptr() as *const u8,
            (data.len() * 2) as u32,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
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

fn to_wide(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
