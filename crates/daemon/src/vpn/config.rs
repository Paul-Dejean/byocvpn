use std::net::SocketAddr;

use base64::{Engine, engine::general_purpose};
use byocvpn_core::error::{ConfigurationError, Result};
use ini::Ini;
use ipnet::IpNet;

pub struct WireguardConfig {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub endpoint: SocketAddr,
    pub ipv4: IpNet,
    pub ipv6: IpNet,
    pub dns_servers: Vec<String>,
}

pub async fn parse_wireguard_config(config_path: &str) -> Result<WireguardConfig> {
    fn require_section<'a>(config: &'a Ini, name: &str) -> Result<&'a ini::Properties> {
        config.section(Some(name)).ok_or_else(|| {
            ConfigurationError::MissingField {
                field: format!("[{}] section", name),
            }
            .into()
        })
    }

    fn require_field<'a>(section: &'a ini::Properties, field: &str) -> Result<&'a str> {
        section.get(field).ok_or_else(|| {
            ConfigurationError::MissingField {
                field: field.to_string(),
            }
            .into()
        })
    }

    fn decode_base64(value: &str, field: &str) -> Result<Vec<u8>> {
        general_purpose::STANDARD.decode(value).map_err(|error| {
            ConfigurationError::InvalidFile {
                reason: format!("Invalid {}: {}", field, error),
            }
            .into()
        })
    }

    fn parse_value<T: std::str::FromStr>(value: &str, field: &str) -> Result<T>
    where
        T::Err: std::fmt::Display,
    {
        value.parse().map_err(|error| {
            ConfigurationError::InvalidFile {
                reason: format!("Invalid {}: {}", field, error),
            }
            .into()
        })
    }

    let config =
        Ini::load_from_file(config_path).map_err(|error| ConfigurationError::InvalidFile {
            reason: format!("Failed to read config file: {}", error),
        })?;

    let interface = require_section(&config, "Interface")?;
    let peer = require_section(&config, "Peer")?;

    let private_key = decode_base64(require_field(interface, "PrivateKey")?, "PrivateKey")?;
    let public_key = decode_base64(require_field(peer, "PublicKey")?, "PublicKey")?;
    let endpoint: SocketAddr = parse_value(require_field(peer, "Endpoint")?, "Endpoint")?;

    let addresses = require_field(interface, "Address")?
        .split(',')
        .map(|address| parse_value::<IpNet>(address.trim(), "Address"))
        .collect::<Result<Vec<_>>>()?;

    let ipv4 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv4())
        .ok_or(ConfigurationError::MissingField {
            field: "IPv4 address".to_string(),
        })?
        .clone();
    let ipv6 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv6())
        .ok_or(ConfigurationError::MissingField {
            field: "IPv6 address".to_string(),
        })?
        .clone();

    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    let dns_servers = parse_domain_name_system_servers_from_interface_section(interface);
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    let dns_servers = Vec::new();

    Ok(WireguardConfig {
        private_key,
        public_key,
        endpoint,
        ipv4,
        ipv6,
        dns_servers,
    })
}

fn parse_domain_name_system_servers_from_interface_section(
    interface_section: &ini::Properties,
) -> Vec<String> {
    let Some(value) = interface_section.get("DNS") else {
        return Vec::new();
    };
    value
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|s| s.to_string())
        .collect()
}
