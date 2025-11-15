#[cfg(target_os = "macos")]
use std::{collections::HashMap, io, process::Command};

use byocvpn_core::error::Result;

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct DomainNameSystemOverrideGuard {
    original_domain_name_system_by_service: HashMap<String, Option<Vec<String>>>,
    domain_name_system_was_applied: bool,
}

#[cfg(target_os = "macos")]
impl DomainNameSystemOverrideGuard {
    /// Applies the given DNS servers to every enabled macOS network service.
    /// Returns a guard that restores the original DNS on Drop.
    pub fn apply_to_all_services(desired_domain_name_system_servers: &[&str]) -> Result<Self> {
        if desired_domain_name_system_servers.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "desired_domain_name_system_servers is empty",
            )
            .into());
        }

        let network_service_names = list_all_enabled_network_services()?;
        println!("Found network services: {:?}", network_service_names);

        let mut original_domain_name_system_by_service = HashMap::new();
        for network_service_name in &network_service_names {
            let current_domain_name_system =
                get_domain_name_system_servers_for_service(network_service_name)?;
            println!(
                "Original DNS for {}: {:?}",
                network_service_name, current_domain_name_system
            );
            original_domain_name_system_by_service
                .insert(network_service_name.clone(), current_domain_name_system);
        }

        println!(
            "Setting new DNS servers: {:?}",
            desired_domain_name_system_servers
        );
        for network_service_name in &network_service_names {
            set_domain_name_system_servers_for_service(
                network_service_name,
                Some(desired_domain_name_system_servers),
            )?;
        }

        Ok(Self {
            original_domain_name_system_by_service,
            domain_name_system_was_applied: true,
        })
    }

    /// Restore immediately (optional; Drop will also restore).
    pub fn restore_now(&mut self) -> io::Result<()> {
        if !self.domain_name_system_was_applied {
            return Ok(());
        }

        println!("Restoring original DNS settings...");

        for (network_service_name, original_option) in &self.original_domain_name_system_by_service
        {
            println!("Restoring DNS for service: {}", network_service_name);
            println!("Original DNS servers: {:?}", original_option);

            match original_option {
                Some(list) if !list.is_empty() => {
                    let as_slices: Vec<&str> = list.iter().map(|s| s.as_str()).collect();
                    println!("Setting DNS servers to: {:?}", as_slices);
                    set_domain_name_system_servers_for_service(
                        network_service_name,
                        Some(&as_slices),
                    )?;
                }
                Some(_) => {
                    println!("Clearing DNS servers (original was empty)");
                    set_domain_name_system_servers_for_service(network_service_name, None)?;
                }
                None => {
                    println!("Clearing DNS servers (original was None)");
                    set_domain_name_system_servers_for_service(network_service_name, None)?;
                }
            }
        }

        // flush_macos_domain_name_system_caches()?;
        self.domain_name_system_was_applied = false;
        println!("DNS restoration completed");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Drop for DomainNameSystemOverrideGuard {
    fn drop(&mut self) {
        let _ = self.restore_now();
    }
}

#[cfg(target_os = "macos")]
fn list_all_enabled_network_services() -> Result<Vec<String>> {
    let mut cmd = Command::new("networksetup");
    cmd.arg("-listallnetworkservices");
    println!("Executing command: {:?}", cmd);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "networksetup -listallnetworkservices failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .into());
    }

    let mut result = Vec::new();
    for (line_index, line) in String::from_utf8_lossy(&output.stdout).lines().enumerate() {
        if line_index == 0 {
            continue;
        } // header
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('*') {
            continue;
        } // disabled
        result.push(trimmed.to_string());
    }
    Ok(result)
}

#[cfg(target_os = "macos")]
fn get_domain_name_system_servers_for_service(
    network_service_name: &str,
) -> io::Result<Option<Vec<String>>> {
    let mut cmd = Command::new("networksetup");
    cmd.arg("-getdnsservers").arg(network_service_name);
    println!("Executing command: {:?}", cmd);
    let output = cmd.output()?;

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
        let t = line.trim();
        if !t.is_empty() {
            servers.push(t.to_string());
        }
    }
    Ok(Some(servers))
}

#[cfg(target_os = "macos")]
fn set_domain_name_system_servers_for_service(
    network_service_name: &str,
    desired_option: Option<&[&str]>,
) -> io::Result<()> {
    let mut cmd = Command::new("networksetup");
    cmd.arg("-setdnsservers").arg(network_service_name);

    match desired_option {
        Some(list) if !list.is_empty() => {
            for server in list {
                cmd.arg(*server);
            }
        }
        _ => {
            cmd.arg("Empty"); // clears DNS
        }
    }

    println!("Executing command: {:?}", cmd);
    let output = cmd.output()?;

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
