/// GCP service-account authentication via the official `google-cloud-auth` library.
///
/// Replaces the previous hand-rolled JWT/RSA/OAuth2 implementation.
use byocvpn_core::error::{ConfigurationError, Result};
use google_cloud_auth::credentials::{CacheableResource, Credentials, service_account};
use http::Extensions;

/// Parse a GCP service-account JSON key string and return official `Credentials`
/// together with the `project_id` extracted from the key.
pub fn credentials_from_service_account_json(json_str: &str) -> Result<(Credentials, String)> {
    let json: serde_json::Value = serde_json::from_str(json_str).map_err(|error| {
        ConfigurationError::InvalidCloudProvider(format!(
            "Invalid GCP service-account JSON: {}",
            error
        ))
    })?;

    let project_id = json["project_id"]
        .as_str()
        .ok_or_else(|| {
            ConfigurationError::InvalidCloudProvider(
                "GCP service-account JSON is missing the 'project_id' field".to_string(),
            )
        })?
        .to_string();

    let credentials = service_account::Builder::new(json)
        .build()
        .map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to build GCP credentials from service-account JSON: {}",
                error
            ))
        })?;

    Ok((credentials, project_id))
}

/// Obtain a short-lived OAuth2 Bearer token from the credentials.
///
/// The official library handles JWT signing, token caching, and renewal.
pub async fn bearer_token(credentials: &Credentials) -> Result<String> {
    let cacheable = credentials
        .headers(Extensions::new())
        .await
        .map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to obtain GCP access token: {}",
                error
            ))
        })?;

    let headers = match cacheable {
        CacheableResource::New { data, .. } => data,
        CacheableResource::NotModified => {
            return Err(ConfigurationError::InvalidCloudProvider(
                "GCP credentials returned NotModified on first token request".to_string(),
            )
            .into());
        }
    };

    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or_else(|| {
            ConfigurationError::InvalidCloudProvider(
                "GCP credentials did not return a Bearer token in the Authorization header"
                    .to_string(),
            )
            .into()
        })
}
