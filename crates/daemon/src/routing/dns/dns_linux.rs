use std::{io, process::Command};

use byocvpn_core::error::{ConfigurationError, Result};
use log::*;

const TUN_INTERFACE_NAME: &str = "tun0";

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

        info!(
            "Setting DNS servers on {} via resolvectl: {:?}",
            TUN_INTERFACE_NAME, new_dns_servers
        );

        let mut set_dns_command = Command::new("resolvectl");
        set_dns_command.arg("dns").arg(TUN_INTERFACE_NAME);
        for server in new_dns_servers {
            set_dns_command.arg(server);
        }

        debug!("Executing: {:?}", set_dns_command);
        let set_dns_output =
            set_dns_command
                .output()
                .map_err(|error| ConfigurationError::DnsConfiguration {
                    reason: format!("failed to run resolvectl dns: {}", error),
                })?;

        if !set_dns_output.status.success() {
            return Err(ConfigurationError::DnsConfiguration {
                reason: format!(
                    "resolvectl dns failed: {}",
                    String::from_utf8_lossy(&set_dns_output.stderr)
                ),
            }
            .into());
        }

        let mut set_domain_command = Command::new("resolvectl");
        set_domain_command
            .arg("domain")
            .arg(TUN_INTERFACE_NAME)
            .arg("~.");

        debug!("Executing: {:?}", set_domain_command);
        let set_domain_output =
            set_domain_command
                .output()
                .map_err(|error| ConfigurationError::DnsConfiguration {
                    reason: format!("failed to run resolvectl domain: {}", error),
                })?;

        if !set_domain_output.status.success() {
            return Err(ConfigurationError::DnsConfiguration {
                reason: format!(
                    "resolvectl domain failed: {}",
                    String::from_utf8_lossy(&set_domain_output.stderr)
                ),
            }
            .into());
        }

        info!("DNS successfully configured on {}", TUN_INTERFACE_NAME);

        Ok(Self {
            dns_override_active: true,
        })
    }

    pub fn restore_previous_dns_configuration(&mut self) -> io::Result<()> {
        if !self.dns_override_active {
            return Ok(());
        }

        info!(
            "Reverting DNS settings on {} via resolvectl",
            TUN_INTERFACE_NAME
        );

        let mut revert_command = Command::new("resolvectl");
        revert_command.arg("revert").arg(TUN_INTERFACE_NAME);

        debug!("Executing: {:?}", revert_command);
        let revert_output = revert_command.output()?;

        if !revert_output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "resolvectl revert failed: {}",
                    String::from_utf8_lossy(&revert_output.stderr)
                ),
            ));
        }

        self.dns_override_active = false;
        info!("DNS restoration completed on {}", TUN_INTERFACE_NAME);
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
