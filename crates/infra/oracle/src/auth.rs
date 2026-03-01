/// OCI API request signing (HTTP Signature, RSA-SHA256).
///
/// OCI signs a canonical "signing string" made from a subset of HTTP headers, then
/// attaches the result in the `Authorization` header. See:
/// https://docs.oracle.com/en-us/iaas/Content/API/Concepts/signingrequests.htm
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::error::{ConfigurationError, Result};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::DecodePrivateKey,
    sha2::Sha256,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256 as Sha256Digest};

/// Credentials needed to authenticate against the OCI API.
#[derive(Clone, Debug)]
pub struct OciCredentials {
    /// OCID of the tenancy, e.g. `ocid1.tenancy.oc1..aaaa…`
    pub tenancy_ocid: String,
    /// OCID of the user, e.g. `ocid1.user.oc1..aaaa…`
    pub user_ocid: String,
    /// RSA key fingerprint as shown in the OCI console, e.g. `aa:bb:cc:…`
    pub fingerprint: String,
    /// PEM-encoded RSA private key (PKCS#8).
    pub private_key_pem: String,
    /// OCI region identifier, e.g. `us-ashburn-1`.
    pub region: String,
}

impl OciCredentials {
    /// The key ID used in the `Authorization` header.
    /// Format: `<tenancy>/<user>/<fingerprint>`
    pub fn key_id(&self) -> String {
        format!(
            "{}/{}/{}",
            self.tenancy_ocid, self.user_ocid, self.fingerprint
        )
    }
}

/// Compute the SHA-256 digest of `body` and return it base64-encoded.
pub fn body_digest(body: &[u8]) -> String {
    let mut hasher = Sha256Digest::new();
    hasher.update(body);
    BASE64.encode(hasher.finalize())
}

/// Build the value for the `Authorization` header for the given request components.
///
/// # Parameters
/// - `method`   — HTTP verb in lower-case (e.g. `"get"`, `"post"`)
/// - `host`     — host header value (e.g. `"iaas.us-ashburn-1.oraclecloud.com"`)
/// - `path`     — request path + query string (e.g. `"/20160918/vcns?compartmentId=…"`)
/// - `date`     — RFC 7231 date string (e.g. `"Fri, 28 Feb 2026 12:00:00 GMT"`)
/// - `body`     — raw request body bytes (empty for GET/DELETE)
/// - `credentials` — the caller's OCI credentials
///
/// Returns the fully-formed `Authorization` header value.
pub fn build_authorization_header(
    method: &str,
    host: &str,
    path: &str,
    date: &str,
    body: Option<&[u8]>,
    credentials: &OciCredentials,
) -> Result<(String, String)> {
    let method_lower = method.to_lowercase();

    // Always-signed headers
    let mut headers_to_sign = vec![
        ("(request-target)", format!("{} {}", method_lower, path)),
        ("host", host.to_string()),
        ("date", date.to_string()),
    ];

    // For POST/PUT include body headers
    let content_sha256 = if let Some(body_bytes) = body {
        let digest = body_digest(body_bytes);
        let content_length = body_bytes.len().to_string();
        headers_to_sign.push(("x-content-sha256", digest.clone()));
        headers_to_sign.push(("content-type", "application/json".to_string()));
        headers_to_sign.push(("content-length", content_length));
        Some(digest)
    } else {
        None
    };

    // Build the signing string: each header on its own line as `name: value`
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

    // Parse the RSA private key (PKCS#8 PEM)
    let private_key = RsaPrivateKey::from_pkcs8_pem(&credentials.private_key_pem).map_err(|e| {
        ConfigurationError::InvalidCloudProvider(format!("Invalid OCI private key: {}", e))
    })?;

    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature_bytes = signing_key
        .try_sign(signing_string.as_bytes())
        .map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("OCI signing failed: {}", e))
        })?
        .to_bytes();

    let signature_b64 = BASE64.encode(signature_bytes);

    let authorization = format!(
        r#"Signature version="1",keyId="{}",algorithm="rsa-sha256",headers="{}",signature="{}""#,
        credentials.key_id(),
        signed_headers,
        signature_b64,
    );

    Ok((authorization, content_sha256.unwrap_or_default()))
}
