use std::net::SocketAddr;

use base64::{Engine, engine::general_purpose};
use byocvpn_core::error::{Error, Result};
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
    // Parse config file
    let config = Ini::load_from_file(config_path)
        .map_err(|e| Error::ConfigParseError(format!("Failed to read config file: {}", e)))?;

    let interface = config
        .section(Some("Interface"))
        .ok_or_else(|| Error::InvalidConfig("[Interface] section missing".to_string()))?;
    let peer = config
        .section(Some("Peer"))
        .ok_or_else(|| Error::InvalidConfig("[Peer] section missing".to_string()))?;

    // Parse private key
    let private_key_str = interface
        .get("PrivateKey")
        .ok_or_else(|| Error::InvalidConfig("PrivateKey missing".to_string()))?;
    let private_key = general_purpose::STANDARD
        .decode(private_key_str)
        .map_err(|e| Error::InvalidConfig(format!("Invalid PrivateKey: {}", e)))?;

    // Parse addresses
    let addresses_str = interface
        .get("Address")
        .ok_or_else(|| Error::InvalidConfig("Address missing".to_string()))?;
    let addresses: Result<Vec<IpNet>> = addresses_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<IpNet>()
                .map_err(|e| Error::InvalidConfig(format!("Invalid address: {}", e)))
        })
        .collect();
    let addresses = addresses?;

    // Parse public key
    let public_key_str = peer
        .get("PublicKey")
        .ok_or_else(|| Error::InvalidConfig("PublicKey missing".to_string()))?;
    let public_key = general_purpose::STANDARD
        .decode(public_key_str)
        .map_err(|e| Error::InvalidConfig(format!("Invalid PublicKey: {}", e)))?;

    // Parse endpoint
    let endpoint_str = peer
        .get("Endpoint")
        .ok_or_else(|| Error::InvalidConfig("Endpoint missing".to_string()))?;
    let endpoint: SocketAddr = endpoint_str
        .parse()
        .map_err(|e| Error::InvalidConfig(format!("Invalid Endpoint: {}", e)))?;

    let ipv4 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv4())
        .ok_or_else(|| Error::InvalidConfig("No IPv4 address found".to_string()))?
        .clone();
    let ipv6 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv6())
        .ok_or_else(|| Error::InvalidConfig("No IPv6 address found".to_string()))?
        .clone();

    #[cfg(target_os = "macos")]
    let dns_servers = parse_domain_name_system_servers_from_interface_section(interface);
    #[cfg(not(target_os = "macos"))]
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
    if let Some(value) = interface_section.get("DNS") {
        value
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    }
}
