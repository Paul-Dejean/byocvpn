use std::net::{SocketAddr, TcpStream};

use base64::{Engine, engine::general_purpose};
use boringtun::x25519::{PublicKey, StaticSecret};
use rand::rngs::OsRng;
use tokio::time::Duration;

pub mod cloud_provider;
pub mod daemon;
pub mod ipc;
mod tunnel;
mod tunnel_manager;
pub mod types;

pub fn generate_keypair() -> (String, String) {
    let private_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    let private_b64 = general_purpose::STANDARD.encode(private_key.to_bytes());
    let public_b64 = general_purpose::STANDARD.encode(public_key.as_bytes());

    (private_b64, public_b64)
}

pub fn can_connect_ipv6() -> bool {
    // Google's IPv6 DNS server (UDP/53, but TCP test works fine)
    let addr: SocketAddr = "[2001:4860:4860::8888]:53".parse().unwrap();

    // Try to open a TCP connection with a 2-second timeout
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
}

pub fn generate_client_config(
    client_private_key: &str,
    server_public_key: &str,
    server_ip_v4: &str,
) -> String {
    return format!(
        r#"[Interface]
PrivateKey = {client_private_key}
Address = 10.66.66.2/24, fd86:ea04:1111::2/128
DNS = 2606:4700:4700::1112, 2606:4700:4700::1002, 1.1.1.2, 1.0.0.2

[Peer]
PublicKey = {server_public_key}
Endpoint = {server_ip_v4}:51820
AllowedIPs = 0.0.0.0/0, ::/0
PersistentKeepalive = 25
"#
    );
}
