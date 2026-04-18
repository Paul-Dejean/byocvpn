use std::{collections::HashMap, io, process::Command};

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

#[derive(Debug)]
pub struct DnsOverrideGuard {
    original_domain_name_system_by_service: HashMap<String, Option<Vec<String>>>,
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

        let network_service_names = list_all_enabled_network_services()?;
        info!("Found network services: {:?}", network_service_names);

        let mut original_domain_name_system_by_service = HashMap::new();
        for network_service_name in &network_service_names {
            let current_domain_name_system = get_dns_servers_for_service(network_service_name)
                .map_err(|error| ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                })?;
            info!(
                "Original DNS for {}: {:?}",
                network_service_name, current_domain_name_system
            );
            original_domain_name_system_by_service
                .insert(network_service_name.clone(), current_domain_name_system);
        }

        info!("Setting new DNS servers: {:?}", new_dns_servers);
        for network_service_name in &network_service_names {
            set_dns_for_service(network_service_name, Some(new_dns_servers)).map_err(|error| {
                ConfigurationError::DnsConfiguration {
                    reason: error.to_string(),
                }
            })?;
        }

        Ok(Self {
            original_domain_name_system_by_service,
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> io::Result<()> {
        if !self.dns_override_active {
            return Ok(());
        }

        info!("Restoring original DNS settings...");

        for (network_service_name, original_option) in &self.original_domain_name_system_by_service
        {
            debug!("Restoring DNS for service: {}", network_service_name);
            debug!("Original DNS servers: {:?}", original_option);

            match original_option {
                Some(list) if !list.is_empty() => {
                    let as_slices: Vec<&str> = list.iter().map(|string| string.as_str()).collect();
                    debug!("Setting DNS servers to: {:?}", as_slices);
                    set_dns_for_service(network_service_name, Some(&as_slices))?;
                }
                Some(_) => {
                    debug!("Clearing DNS servers (original was empty)");
                    set_dns_for_service(network_service_name, None)?;
                }
                None => {
                    debug!("Clearing DNS servers (original was None)");
                    set_dns_for_service(network_service_name, None)?;
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

fn list_all_enabled_network_services() -> Result<Vec<String>> {
    let mut command = Command::new("networksetup");
    command.arg("-listallnetworkservices");
    debug!("Executing command: {:?}", command);
    let output = command
        .output()
        .map_err(|error| ConfigurationError::DnsConfiguration {
            reason: format!("failed to run networksetup: {}", error),
        })?;
    if !output.status.success() {
        return Err(ConfigurationError::DnsConfiguration {
            reason: format!(
                "networksetup -listallnetworkservices failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }
        .into());
    }

    let mut result = Vec::new();
    for (line_index, line) in String::from_utf8_lossy(&output.stdout).lines().enumerate() {
        if line_index == 0 {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('*') {
            continue;
        }
        result.push(trimmed.to_string());
    }
    Ok(result)
}

fn get_dns_servers_for_service(network_service_name: &str) -> io::Result<Option<Vec<String>>> {
    let mut command = Command::new("networksetup");
    command.arg("-getdnsservers").arg(network_service_name);
    debug!("Executing command: {:?}", command);
    let output = command.output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "networksetup -getdnsservers {} failed: {}",
                network_service_name,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("aren't any DNS Servers set") {
        return Ok(None);
    }

    let mut servers = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            servers.push(trimmed.to_string());
        }
    }
    Ok(Some(servers))
}

fn set_dns_for_service(
    network_service_name: &str,
    desired_option: Option<&[&str]>,
) -> io::Result<()> {
    let mut command = Command::new("networksetup");
    command.arg("-setdnsservers").arg(network_service_name);

    match desired_option {
        Some(list) if !list.is_empty() => {
            for server in list {
                command.arg(*server);
            }
        }
        _ => {
            command.arg("Empty");
        }
    }

    debug!("Executing command: {:?}", command);
    let output = command.output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "networksetup -setdnsservers {} failed: {}",
                network_service_name,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}
