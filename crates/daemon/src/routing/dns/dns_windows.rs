use std::{
    ffi::OsStr,
    io::{Error as IoError, Result as IoResult},
    iter::once,
    mem::{size_of_val, transmute},
    os::windows::ffi::OsStrExt,
    ptr::{null, null_mut},
};

use byocvpn_core::error::{Result, TunnelError};
use log::{debug, info, warn};
use windows_sys::Win32::{
    Foundation::FreeLibrary,
    Security::SECURITY_ATTRIBUTES,
    System::{
        LibraryLoader::{GetProcAddress, LoadLibraryW},
        Registry::{
            HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD, REG_MULTI_SZ,
            REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteKeyExW,
            RegSetValueExW,
        },
    },
};

const NRPT_POLICY_KEY: &str =
    r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig";
const NRPT_RULE_SUBKEY: &str = "{ByocVPN-DNS-Override}";
const NRPT_VERSION: u32 = 2;
const NRPT_CONFIG_DNS_SERVERS: u32 = 0x08;
const ERROR_FILE_NOT_FOUND: u32 = 2;

type DnsFlushFn = unsafe extern "system" fn() -> i32;

#[derive(Debug)]
pub struct DnsOverrideGuard {
    dns_override_active: bool,
}

impl DnsOverrideGuard {
    pub fn override_system_dns(new_dns_servers: &[&str]) -> Result<Self> {
        if new_dns_servers.is_empty() {
            return Err(TunnelError::NoDnsServers.into());
        }

        add_nrpt_rule(new_dns_servers).map_err(TunnelError::DnsRegistry)?;

        flush_dns_resolver_cache();
        info!("NRPT DNS rule applied for servers: {new_dns_servers:?}");

        Ok(Self {
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> IoResult<()> {
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
            warn!("Failed to restore DNS on drop: {error}");
        }
    }
}

fn add_nrpt_rule(dns_servers: &[&str]) -> IoResult<()> {
    let key_path_w = to_wide(&build_rule_key_path());

    let mut key = null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            key_path_w.as_ptr(),
            0,
            null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            null::<SECURITY_ATTRIBUTES>(),
            &raw mut key,
            null_mut(),
        )
    };
    if result != 0 {
        return Err(build_registry_error(result));
    }

    let write_result = write_nrpt_values(key, dns_servers);
    unsafe { RegCloseKey(key) };
    write_result
}

fn write_nrpt_values(key: HKEY, dns_servers: &[&str]) -> IoResult<()> {
    set_registry_dword(key, "Version", NRPT_VERSION)?;
    set_registry_dword(key, "ConfigOptions", NRPT_CONFIG_DNS_SERVERS)?;
    set_registry_multi_sz(key, "Name", &["."])?;
    set_registry_sz(key, "GenericDNSServers", &dns_servers.join(";"))
}

fn remove_nrpt_rule() -> IoResult<()> {
    let key_path_w = to_wide(&build_rule_key_path());

    let result = unsafe { RegDeleteKeyExW(HKEY_LOCAL_MACHINE, key_path_w.as_ptr(), 0, 0) };
    if result != 0 && result != ERROR_FILE_NOT_FOUND {
        return Err(build_registry_error(result));
    }
    Ok(())
}

fn build_rule_key_path() -> String {
    format!("{NRPT_POLICY_KEY}\\{NRPT_RULE_SUBKEY}")
}

fn set_registry_dword(key: HKEY, name: &str, value: u32) -> IoResult<()> {
    let name_w = to_wide(name);
    let result = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_DWORD,
            (&raw const value).cast::<u8>(),
            count_bytes(&value),
        )
    };
    if result != 0 {
        return Err(build_registry_error(result));
    }
    Ok(())
}

fn set_registry_sz(key: HKEY, name: &str, value: &str) -> IoResult<()> {
    let name_w = to_wide(name);
    let value_w = to_wide(value);
    let result = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_SZ,
            value_w.as_ptr().cast::<u8>(),
            count_bytes(value_w.as_slice()),
        )
    };
    if result != 0 {
        return Err(build_registry_error(result));
    }
    Ok(())
}

fn set_registry_multi_sz(key: HKEY, name: &str, values: &[&str]) -> IoResult<()> {
    let name_w = to_wide(name);
    let mut data: Vec<u16> = Vec::new();
    for value in values {
        data.extend(value.encode_utf16());
        data.push(0);
    }
    data.push(0);

    let result = unsafe {
        RegSetValueExW(
            key,
            name_w.as_ptr(),
            0,
            REG_MULTI_SZ,
            data.as_ptr().cast::<u8>(),
            count_bytes(data.as_slice()),
        )
    };
    if result != 0 {
        return Err(build_registry_error(result));
    }
    Ok(())
}

fn flush_dns_resolver_cache() {
    let library = unsafe { LoadLibraryW(to_wide("dnsapi.dll").as_ptr()) };
    if library.is_null() {
        debug!("Failed to load dnsapi.dll");
        return;
    }

    let flush_address =
        unsafe { GetProcAddress(library, c"DnsFlushResolverCache".as_ptr().cast()) };
    if let Some(address) = flush_address {
        let flush: DnsFlushFn = unsafe { transmute(address) };
        unsafe { flush() };
        debug!("DNS resolver cache flushed");
    }

    unsafe { FreeLibrary(library) };
}

fn build_registry_error(code: u32) -> IoError {
    IoError::from_raw_os_error(i32::try_from(code).unwrap_or(i32::MAX))
}

fn count_bytes<T>(value: &T) -> u32
where
    T: ?Sized,
{
    u32::try_from(size_of_val(value)).unwrap_or(u32::MAX)
}

fn to_wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(once(0)).collect()
}
