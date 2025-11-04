use base64::{Engine, engine::general_purpose};
use boringtun::x25519::{PublicKey, StaticSecret};
use rand::rngs::OsRng;

pub fn generate_keypair() -> (String, String) {
    let private_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    let private_b64 = general_purpose::STANDARD.encode(private_key.to_bytes());
    let public_b64 = general_purpose::STANDARD.encode(public_key.as_bytes());

    (private_b64, public_b64)
}
