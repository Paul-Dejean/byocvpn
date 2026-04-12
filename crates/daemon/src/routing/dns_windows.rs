#[cfg(windows)]
use std::{collections::HashMap, io, process::Command};

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

#[cfg(windows)]
#[derive(Debug)]
enum OriginalDnsState {
    Dhcp,
    Static(Vec<String>),
    Unconfigured,
}

#[cfg(windows)]
#[derive(Debug)]
pub struct DomainNameSystemOverrideGuard {
    original_state_by_interface: HashMap<String, OriginalDnsState>,
    domain_name_system_was_applied: bool,
}

#[cfg(windows)]
impl DomainNameSystemOverrideGuard {
    pub fn apply_to_all_services(desired_domain_name_system_servers: &[&str]) -> Result<Self> {
        if desired_domain_name_system_servers.is_empty() {
            return Err(ConfigurationError::DnsConfiguration {
                reason: "desired DNS servers list is empty".to_string(),
            }
            .into());
        }

        let interface_names = list_connected_interfaces()?;
        info!("Found network interfaces: {:?}", interface_names);

        let mut original_state_by_interface = HashMap::new();
        for interface_name in &interface_names {
            let state = get_dns_state_for_interface(interface_name).map_err(|error| {
                ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                }
            })?;
            info!("Original DNS for {}: {:?}", interface_name, state);
            original_state_by_interface.insert(interface_name.clone(), state);
        }

        info!(
            "Setting new DNS servers: {:?}",
            desired_domain_name_system_servers
        );
        for interface_name in &interface_names {
            set_static_dns_for_interface(interface_name, desired_domain_name_system_servers)
                .map_err(|error| ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                })?;
        }

        Ok(Self {
            original_state_by_interface,
            domain_name_system_was_applied: true,
        })
    }

    pub fn restore_now(&mut self) -> io::Result<()> {
        if !self.domain_name_system_was_applied {
            return Ok(());
        }

        info!("Restoring original DNS settings...");
        for (interface_name, original_state) in &self.original_state_by_interface {
            info!("Restoring DNS for interface: {}", interface_name);
            match original_state {
                OriginalDnsState::Dhcp | OriginalDnsState::Unconfigured => {
                    restore_dhcp_dns(interface_name)?;
                }
                OriginalDnsState::Static(servers) => {
                    let as_refs: Vec<&str> = servers.iter().map(|s| s.as_str()).collect();
                    set_static_dns_for_interface(interface_name, &as_refs)?;
                }
            }
        }

        self.domain_name_system_was_applied = false;
        info!("DNS restoration completed");
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for DomainNameSystemOverrideGuard {
    fn drop(&mut self) {
        let _ = self.restore_now();
    }
}

#[cfg(windows)]
fn list_connected_interfaces() -> Result<Vec<String>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object -ExpandProperty Name",
        ])
        .output()
        .map_err(|error| ConfigurationError::DnsConfiguration {
            reason: format!("failed to run PowerShell to list interfaces: {}", error),
        })?;

    if !output.status.success() {
        return Err(ConfigurationError::DnsConfiguration {
            reason: format!(
                "Get-NetAdapter failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }
        .into());
    }

    let interfaces = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(interfaces)
}

#[cfg(windows)]
fn get_dns_state_for_interface(interface_name: &str) -> io::Result<OriginalDnsState> {
    let output = Command::new("netsh")
        .args([
            "interface",
            "ip",
            "show",
            "dns",
            &format!("name={}", interface_name),
        ])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);

    if text.contains("DNS servers configured through DHCP") {
        return Ok(OriginalDnsState::Dhcp);
    }

    if text.contains("Statically Configured DNS Servers") {
        let servers = parse_dns_server_ips_from_netsh_output(&text);
        return Ok(OriginalDnsState::Static(servers));
    }

    Ok(OriginalDnsState::Unconfigured)
}

#[cfg(windows)]
fn parse_dns_server_ips_from_netsh_output(text: &str) -> Vec<String> {
    let mut servers = Vec::new();
    let mut in_dns_block = false;

    for line in text.lines() {
        if line.contains("DNS Servers") || line.contains("DNS servers") {
            in_dns_block = true;
            if let Some(ip) = extract_ip_from_line(line) {
                servers.push(ip);
            }
        } else if in_dns_block {
            let trimmed = line.trim();
            if trimmed.is_empty() || line.contains(':') {
                in_dns_block = false;
            } else if let Some(ip) = extract_ip_from_line(line) {
                servers.push(ip);
            }
        }
    }

    servers
}

#[cfg(windows)]
fn extract_ip_from_line(line: &str) -> Option<String> {
    for token in line.split_whitespace() {
        let is_ipv4 = token.contains('.') && token.split('.').count() == 4;
        let is_ipv6 = token.contains(':');
        if is_ipv4 || is_ipv6 {
            return Some(token.to_string());
        }
    }
    None
}

#[cfg(windows)]
fn set_static_dns_for_interface(interface_name: &str, servers: &[&str]) -> io::Result<()> {
    if servers.is_empty() {
        return Ok(());
    }

    let output = Command::new("netsh")
        .args([
            "interface",
            "ip",
            "set",
            "dns",
            &format!("name={}", interface_name),
            "static",
            servers[0],
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "netsh set dns failed for {}: {}",
                interface_name,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    for (index, server) in servers[1..].iter().enumerate() {
        let output = Command::new("netsh")
            .args([
                "interface",
                "ip",
                "add",
                "dns",
                &format!("name={}", interface_name),
                &format!("addr={}", server),
                &format!("index={}", index + 2),
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "netsh add dns failed for {}: {}",
                    interface_name,
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
    }

    Ok(())
}

#[cfg(windows)]
fn restore_dhcp_dns(interface_name: &str) -> io::Result<()> {
    let output = Command::new("netsh")
        .args([
            "interface",
            "ip",
            "set",
            "dns",
            &format!("name={}", interface_name),
            "dhcp",
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "netsh restore dhcp dns failed for {}: {}",
                interface_name,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}
