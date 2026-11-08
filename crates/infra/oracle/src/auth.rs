use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::error::{CredentialsError, Result};
use reqwest::Method;
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    sha2::Sha256,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256 as Sha256Digest};

use crate::credentials::OracleCredentials;

pub struct SignedRequestHeaders {
    pub authorization: String,
    pub content_sha256: Option<String>,
}

pub fn build_signed_headers(
    credentials: &OracleCredentials,
    method: &Method,
    host: &str,
    path: &str,
    date: &str,
    body: Option<&[u8]>,
) -> Result<SignedRequestHeaders> {
    let request_target = format!("{} {path}", method.as_str().to_lowercase());
    let mut headers_to_sign = vec![
        ("(request-target)", request_target),
        ("host", host.to_string()),
        ("date", date.to_string()),
    ];

    let content_sha256 = body.map(|body_bytes| {
        let digest = compute_body_digest(body_bytes);
        headers_to_sign.push(("x-content-sha256", digest.clone()));
        headers_to_sign.push(("content-type", "application/json".to_string()));
        headers_to_sign.push(("content-length", body_bytes.len().to_string()));
        digest
    });

    let signing_string = headers_to_sign
        .iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    let signed_header_names = headers_to_sign
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(" ");

    let signature = sign(credentials, signing_string.as_bytes())?;
    let authorization = format!(
        r#"Signature version="1",keyId="{}",algorithm="rsa-sha256",headers="{signed_header_names}",signature="{signature}""#,
        build_key_id(credentials),
    );

    Ok(SignedRequestHeaders {
        authorization,
        content_sha256,
    })
}

fn build_key_id(credentials: &OracleCredentials) -> String {
    format!(
        "{}/{}/{}",
        credentials.tenancy_ocid, credentials.user_ocid, credentials.fingerprint
    )
}

fn compute_body_digest(body: &[u8]) -> String {
    let mut hasher = Sha256Digest::new();
    hasher.update(body);
    BASE64.encode(hasher.finalize())
}

fn sign(credentials: &OracleCredentials, message: &[u8]) -> Result<String> {
    let private_key =
        RsaPrivateKey::from_pkcs8_pem(&credentials.private_key_pem).map_err(|error| {
            CredentialsError::InvalidPrivateKey {
                reason: error.to_string(),
            }
        })?;
    let signature = SigningKey::<Sha256>::new(private_key)
        .try_sign(message)
        .map_err(|error| CredentialsError::SigningFailed {
            reason: error.to_string(),
        })?;
    Ok(BASE64.encode(signature.to_bytes()))
}
