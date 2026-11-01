use byocvpn_core::cloud_provider::CloudProviderName;
use byocvpn_core::error::{Error, NetworkProvisioningError, Result};
use google_cloud_auth::credentials::Credentials;
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::auth::fetch_bearer_token;
use log::{debug, error};

pub struct GcpClient {
    http_client: HttpClient,
    credentials: Credentials,
    pub project_id: String,
}

impl GcpClient {
    pub fn new(credentials: Credentials, project_id: String) -> Self {
        Self {
            http_client: HttpClient::new(),
            credentials,
            project_id,
        }
    }

    pub fn build_compute_base_url(&self) -> String {
        format!(
            "https://compute.googleapis.com/compute/v1/projects/{}",
            self.project_id
        )
    }

    async fn get_bearer_token(&self) -> Result<String> {
        fetch_bearer_token(&self.credentials).await
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        debug!("[GCP] GET {url}");
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP GET {url} failed: {error}"),
            })?;
        parse_response("GET", url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[GCP] POST {url}");
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP POST {url} failed: {error}"),
            })?;
        parse_response("POST", url, response).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[GCP] PATCH {url}");
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .patch(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP PATCH {url} failed: {error}"),
            })?;
        parse_response("PATCH", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("[GCP] DELETE {url}");
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .delete(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP DELETE {url} failed: {error}"),
            })?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(build_request_error("DELETE", url, status, &body))
    }
}

async fn parse_response<T: DeserializeOwned>(
    method: &str,
    url: &str,
    response: Response,
) -> Result<T> {
    let status = response.status();
    let body =
        response
            .text()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to read GCP response body: {error}"),
            })?;

    if status.is_success() {
        debug!("[GCP] {method} {url} → {status}");
        let json_str = if body.is_empty() {
            "null"
        } else {
            body.as_str()
        };
        from_str(json_str).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to parse GCP JSON: {error} — body: {body}"),
            }
            .into()
        })
    } else if status == StatusCode::NOT_FOUND {
        Err(NetworkProvisioningError::ResourceNotFound {
            url: url.to_string(),
        }
        .into())
    } else {
        error!("[GCP] {method} {url} → {status} — {body}");
        Err(build_request_error(method, url, status, &body))
    }
}

#[derive(Deserialize)]
struct GcpErrorResponse {
    error: Option<GcpErrorBody>,
}

#[derive(Deserialize)]
struct GcpErrorBody {
    message: Option<String>,
    status: Option<String>,
    #[serde(default)]
    errors: Vec<GcpErrorDetail>,
}

#[derive(Deserialize)]
struct GcpErrorDetail {
    reason: Option<String>,
}

fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
    let error_body = from_str::<GcpErrorResponse>(body)
        .ok()
        .and_then(|response| response.error);
    let code = error_body.as_ref().and_then(|error_body| {
        error_body.status.clone().or_else(|| {
            error_body
                .errors
                .first()
                .and_then(|detail| detail.reason.clone())
        })
    });
    let message = error_body.and_then(|error_body| error_body.message);
    Error::cloud_api_rejected(
        CloudProviderName::Gcp,
        format!("{method} {url}"),
        status.as_u16(),
        code,
        message,
        body,
    )
}
