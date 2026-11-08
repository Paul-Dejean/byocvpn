use google_cloud_auth::credentials::{CacheableResource, Credentials, service_account};
use http::Extensions;

use crate::error::{GcpError, Result};

pub fn parse_credentials_from_service_account_json(
    json_str: &str,
) -> Result<(Credentials, String)> {
    let json: serde_json::Value =
        serde_json::from_str(json_str).map_err(GcpError::InvalidServiceAccountJson)?;

    let project_id = json["project_id"]
        .as_str()
        .ok_or(GcpError::MissingProjectId)?
        .to_string();

    let credentials = service_account::Builder::new(json).build()?;

    Ok((credentials, project_id))
}

pub async fn fetch_bearer_token(credentials: &Credentials) -> Result<String> {
    let headers = match credentials.headers(Extensions::new()).await? {
        CacheableResource::New { data, .. } => data,
        CacheableResource::NotModified => return Err(GcpError::MissingBearerToken),
    };

    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|header_value| header_value.strip_prefix("Bearer "))
        .map(ToString::to_string)
        .ok_or(GcpError::MissingBearerToken)
}
