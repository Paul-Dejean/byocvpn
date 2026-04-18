use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::error::{CredentialsError, Result};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    sha2::Sha256,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256 as Sha256Digest};

#[derive(Clone, Debug)]
pub struct OciCredentials {
    pub tenancy_ocid: String,
    pub user_ocid: String,
    pub fingerprint: String,
    pub private_key_pem: String,
    pub region: String,
}

impl OciCredentials {
    pub fn build_key_id(&self) -> String {
        format!(
            "{}/{}/{}",
            self.tenancy_ocid, self.user_ocid, self.fingerprint
        )
    }
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

pub fn compute_body_digest(body: &[u8]) -> String {
    let mut hasher = Sha256Digest::new();
    hasher.update(body);
    BASE64.encode(hasher.finalize())
}

pub fn build_authorization_header(
    method: HttpMethod,
    host: &str,
    path: &str,
    date: &str,
    body: Option<&[u8]>,
    credentials: &OciCredentials,
) -> Result<(String, String)> {
    let mut headers_to_sign = vec![
        ("(request-target)", format!("{} {}", method, path)),
        ("host", host.to_string()),
        ("date", date.to_string()),
    ];

    let content_sha256 = if let Some(body_bytes) = body {
        let digest = compute_body_digest(body_bytes);
        let content_length = body_bytes.len().to_string();
        headers_to_sign.push(("x-content-sha256", digest.clone()));
        headers_to_sign.push(("content-type", "application/json".to_string()));
        headers_to_sign.push(("content-length", content_length));
        Some(digest)
    } else {
        None
    };

    let signing_string = headers_to_sign
        .iter()
        .map(|(name, value)| format!("{}: {}", name, value))
        .collect::<Vec<_>>()
        .join("\n");

    let signed_headers = headers_to_sign
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(" ");

    let private_key =
        RsaPrivateKey::from_pkcs8_pem(&credentials.private_key_pem).map_err(|error| {
            CredentialsError::InvalidPrivateKey {
                reason: error.to_string(),
            }
        })?;

    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature_bytes = signing_key
        .try_sign(signing_string.as_bytes())
        .map_err(|error| CredentialsError::SigningFailed {
            reason: error.to_string(),
        })?
        .to_bytes();

    let signature_b64 = BASE64.encode(signature_bytes);

    let authorization = format!(
        r#"Signature version="1",keyId="{}",algorithm="rsa-sha256",headers="{}",signature="{}""#,
        credentials.build_key_id(),
        signed_headers,
        signature_b64,
    );

    Ok((authorization, content_sha256.unwrap_or_default()))
}
