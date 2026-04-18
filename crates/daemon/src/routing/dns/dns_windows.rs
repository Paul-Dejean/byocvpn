use std::{collections::HashMap, io, process::Command};

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

#[derive(Debug)]
enum OriginalDnsState {
    Dhcp,
    Static(Vec<String>),
    Unconfigured,
}

#[derive(Debug)]
pub struct DnsOverrideGuard {
    previous_dns_configuration: HashMap<String, OriginalDnsState>,
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

        let interface_names: Vec<String> = list_connected_interfaces()?
            .into_iter()
            .filter(|name| name != "byocvpn")
            .collect();
        info!(
            "Storing previous dns configuration for network interfaces: {:?}",
            interface_names
        );

        let mut previous_dns_configuration = HashMap::new();
        for interface_name in &interface_names {
            let state = get_dns_state_for_interface(interface_name).map_err(|error| {
                ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                }
            })?;
            debug!("Original DNS for {}: {:?}", interface_name, state);
            previous_dns_configuration.insert(interface_name.clone(), state);
        }

        info!("Setting new DNS servers: {:?}", new_dns_servers);
        for interface_name in &interface_names {
            set_dns_servers_for_interface(interface_name, new_dns_servers).map_err(|error| {
                ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                }
            })?;
        }

        Ok(Self {
            previous_dns_configuration,
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> io::Result<()> {
        if !self.dns_override_active {
            return Ok(());
        }

        info!("Restoring original DNS settings...");
        for (interface_name, original_state) in &self.previous_dns_configuration {
            info!("Restoring DNS for interface: {}", interface_name);
            match original_state {
                OriginalDnsState::Dhcp | OriginalDnsState::Unconfigured => {
                    restore_dhcp_dns(interface_name)?;
                }
                OriginalDnsState::Static(servers) => {
                    let as_refs: Vec<&str> = servers.iter().map(|string| string.as_str()).collect();
                    set_dns_servers_for_interface(interface_name, &as_refs)?;
                }
            }
        }

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

fn set_dns_servers_for_interface(interface_name: &str, servers: &[&str]) -> io::Result<()> {
    if servers.is_empty() {
        return Ok(());
    }

    // Split servers into IPv4 and IPv6
    let ipv4_servers: Vec<&str> = servers
        .iter()
        .copied()
        .filter(|s| !s.contains(':'))
        .collect();
    let ipv6_servers: Vec<&str> = servers
        .iter()
        .copied()
        .filter(|s| s.contains(':'))
        .collect();

    // Set IPv4 DNS
    if let Some(first) = ipv4_servers.first() {
        let output = Command::new("netsh")
            .args([
                "interface",
                "ip",
                "set",
                "dns",
                &format!("name={}", interface_name),
                "static",
                first,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("netsh ip set dns failed for {}: {}", interface_name, stderr);
        }

        for (index, server) in ipv4_servers[1..].iter().enumerate() {
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
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("netsh ip add dns failed for {}: {}", interface_name, stderr);
            }
        }
    }

    // Set IPv6 DNS
    if let Some(first) = ipv6_servers.first() {
        let output = Command::new("netsh")
            .args([
                "interface",
                "ipv6",
                "set",
                "dns",
                &format!("name={}", interface_name),
                "static",
                first,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "netsh ipv6 set dns failed for {}: {}",
                interface_name, stderr
            );
        }

        for (index, server) in ipv6_servers[1..].iter().enumerate() {
            let output = Command::new("netsh")
                .args([
                    "interface",
                    "ipv6",
                    "add",
                    "dns",
                    &format!("name={}", interface_name),
                    &format!("addr={}", server),
                    &format!("index={}", index + 2),
                ])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(
                    "netsh ipv6 add dns failed for {}: {}",
                    interface_name, stderr
                );
            }
        }
    }

    Ok(())
}

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
