use byocvpn_core::error::{CredentialsError, Result};
use google_cloud_auth::credentials::{CacheableResource, Credentials, service_account};
use http::Extensions;

pub fn parse_credentials_from_service_account_json(
    json_str: &str,
) -> Result<(Credentials, String)> {
    let json: serde_json::Value = serde_json::from_str(json_str).map_err(|error| {
        CredentialsError::InvalidFormat {
            reason: format!("Invalid GCP service-account JSON: {}", error),
        }
    })?;

    let project_id = json["project_id"]
        .as_str()
        .ok_or_else(|| CredentialsError::MissingField {
            field: "project_id".to_string(),
        })?
        .to_string();

    let credentials = service_account::Builder::new(json)
        .build()
        .map_err(|error| CredentialsError::InvalidFormat {
            reason: format!(
                "Failed to build GCP credentials from service-account JSON: {}",
                error
            ),
        })?;

    Ok((credentials, project_id))
}

pub async fn fetch_bearer_token(credentials: &Credentials) -> Result<String> {
    let cacheable = credentials
        .headers(Extensions::new())
        .await
        .map_err(|error| CredentialsError::TokenAcquisitionFailed {
            provider: "GCP".to_string(),
            reason: error.to_string(),
        })?;

    let headers = match cacheable {
        CacheableResource::New { data, .. } => data,
        CacheableResource::NotModified => {
            return Err(CredentialsError::MissingTokenInResponse {
                provider: "GCP".to_string(),
            }
            .into());
        }
    };

    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or_else(|| {
            CredentialsError::MissingTokenInResponse {
                provider: "GCP".to_string(),
            }
            .into()
        })
}
