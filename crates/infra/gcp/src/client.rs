use byocvpn_core::error::{NetworkProvisioningError, Result};
use google_cloud_auth::credentials::Credentials;
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::auth::fetch_bearer_token;
use log::*;

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
        debug!("[GCP] GET {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP GET {} failed: {}", url, error),
            })?;
        parse_response("GET", url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[GCP] POST {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP POST {} failed: {}", url, error),
            })?;
        parse_response("POST", url, response).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[GCP] PATCH {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .patch(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP PATCH {} failed: {}", url, error),
            })?;
        parse_response("PATCH", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("[GCP] DELETE {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .delete(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP DELETE {} failed: {}", url, error),
            })?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("GCP DELETE {} returned {}: {}", url, status, body),
        }
        .into())
    }
}

async fn parse_response<T: DeserializeOwned>(method: &str, url: &str, response: Response) -> Result<T> {
    let status = response.status();
    let body =
        response
            .text()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to read GCP response body: {}", error),
            })?;

    if status.is_success() {
        debug!("[GCP] {} {} → {}", method, url, status);
        let json_str = if body.is_empty() { "null" } else { body.as_str() };
        from_str(json_str).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to parse GCP JSON: {} — body: {}", error, body),
            }
            .into()
        })
    } else if status == StatusCode::NOT_FOUND {
        Err(NetworkProvisioningError::ResourceNotFound { url: url.to_string() }.into())
    } else {
        error!("[GCP] {} {} → {} — {}", method, url, status, body);
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("GCP {} {} error {}: {}", method, url, status, body),
        }
        .into())
    }
}
