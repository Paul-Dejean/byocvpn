use base64::{Engine, engine::general_purpose};
use boringtun::x25519::{PublicKey, StaticSecret};
use rand::{Rng, rng};

const PRIVATE_KEY_LENGTH: usize = 32;

pub fn generate_keypair() -> (String, String) {
    let mut private_key_bytes = [0u8; PRIVATE_KEY_LENGTH];
    rng().fill_bytes(&mut private_key_bytes);
    let private_key = StaticSecret::from(private_key_bytes);
    let public_key = PublicKey::from(&private_key);

    let private_b64 = general_purpose::STANDARD.encode(private_key.to_bytes());
    let public_b64 = general_purpose::STANDARD.encode(public_key.as_bytes());

    (private_b64, public_b64)
}
