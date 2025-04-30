use base64::{Engine, engine::general_purpose};
use boringtun::x25519::{PublicKey, StaticSecret};
use rand::rngs::OsRng;

pub mod cloud_provider;
pub mod daemon;
pub mod ipc;
mod tunnel;
mod types;

pub fn generate_keypair() -> (String, String) {
    let private_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    let private_b64 = general_purpose::STANDARD.encode(private_key.to_bytes());
    let public_b64 = general_purpose::STANDARD.encode(public_key.as_bytes());

    (private_b64, public_b64)
}

pub fn generate_client_config(
    client_private_key: &str,
    server_public_key: &str,
    server_ip: &str,
) -> String {
    format!(
        r#"[Interface]
PrivateKey = {client_private_key}
Address = 10.66.66.2/24
DNS = 1.1.1.1

[Peer]
PublicKey = {server_public_key}
Endpoint = {server_ip}:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
"#
    )
}
