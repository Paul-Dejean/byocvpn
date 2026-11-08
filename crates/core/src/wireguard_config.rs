use std::{error::Error as StdError, net::SocketAddr, path::Path, str::FromStr};

use base64::{Engine, engine::general_purpose};
use ini::{Ini, Properties};
use ipnet::IpNet;
use log::debug;

use crate::error::{ConfigurationError, Result};

pub struct WireguardConfig {
    pub instance_id: String,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub server_endpoint: SocketAddr,
    pub private_ipv4: IpNet,
    pub private_ipv6: IpNet,
    pub dns_servers: Vec<String>,
}

pub async fn parse_wireguard_config(config_path: &str) -> Result<WireguardConfig> {
    let instance_id = Path::new(config_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToString::to_string)
        .ok_or(ConfigurationError::InvalidValue {
            field: "filename",
            reason: "unable to extract the instance id",
        })?;

    let config =
        Ini::load_from_file(config_path).map_err(ConfigurationError::WireguardConfigFile)?;

    let interface = require_section(&config, "Interface")?;
    let peer = require_section(&config, "Peer")?;

    let private_key = decode_base64(require_field(interface, "PrivateKey")?, "PrivateKey")?;
    let public_key = decode_base64(require_field(peer, "PublicKey")?, "PublicKey")?;
    let server_endpoint: SocketAddr = parse_value(require_field(peer, "Endpoint")?, "Endpoint")?;

    let addresses = require_field(interface, "Address")?
        .split(',')
        .map(|address| parse_value::<IpNet>(address.trim(), "Address"))
        .collect::<Result<Vec<_>>>()?;

    let private_ipv4 = *addresses
        .iter()
        .find(|ip| ip.addr().is_ipv4())
        .ok_or_else(|| ConfigurationError::MissingField {
            field: "IPv4 address".to_string(),
        })?;
    let private_ipv6 = *addresses
        .iter()
        .find(|ip| ip.addr().is_ipv6())
        .ok_or_else(|| ConfigurationError::MissingField {
            field: "IPv6 address".to_string(),
        })?;

    let dns_servers = parse_dns_servers_from_interface_section(interface);

    debug!(
        "Parsed WireGuard config: server_endpoint={server_endpoint}, private_ipv4={private_ipv4}, private_ipv6={private_ipv6}, dns_servers={dns_servers:?}"
    );

    Ok(WireguardConfig {
        instance_id,
        private_key,
        public_key,
        server_endpoint,
        private_ipv4,
        private_ipv6,
        dns_servers,
    })
}

fn require_section<'a>(config: &'a Ini, name: &str) -> Result<&'a Properties> {
    config.section(Some(name)).ok_or_else(|| {
        ConfigurationError::MissingField {
            field: format!("[{name}] section"),
        }
        .into()
    })
}

fn require_field<'a>(section: &'a Properties, field: &str) -> Result<&'a str> {
    section.get(field).ok_or_else(|| {
        ConfigurationError::MissingField {
            field: field.to_string(),
        }
        .into()
    })
}

fn decode_base64(value: &str, field: &'static str) -> Result<Vec<u8>> {
    general_purpose::STANDARD.decode(value).map_err(|error| {
        ConfigurationError::InvalidWireguardField {
            field,
            source: Box::new(error),
        }
        .into()
    })
}

fn parse_value<T: FromStr>(value: &str, field: &'static str) -> Result<T>
where
    T::Err: StdError + Send + Sync + 'static,
{
    value.parse().map_err(|error| {
        ConfigurationError::InvalidWireguardField {
            field,
            source: Box::new(error),
        }
        .into()
    })
}

fn parse_dns_servers_from_interface_section(interface_section: &Properties) -> Vec<String> {
    let Some(value) = interface_section.get("DNS") else {
        return Vec::new();
    };
    value
        .split(|character: char| character == ',' || character.is_whitespace())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}
