use byocvpn_core::error::{ConfigurationError, Result};
use google_cloud_auth::credentials::Credentials;
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::Serialize;
use serde_json::Value;

use crate::auth::bearer_token;

/// Thin HTTP client that authenticates every request using the official
/// `google-cloud-auth` library (handles JWT signing, token caching, renewal).
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

    /// Base URL for the Compute Engine v1 REST API scoped to this project.
    pub fn compute_base_url(&self) -> String {
        format!(
            "https://compute.googleapis.com/compute/v1/projects/{}",
            self.project_id
        )
    }

    /// Obtain an OAuth2 Bearer token (cached + auto-renewed by google-cloud-auth).
    async fn get_bearer_token(&self) -> Result<String> {
        bearer_token(&self.credentials).await
    }

    pub async fn get(&self, url: &str) -> Result<Value> {
        eprintln!("[GCP] GET {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| {
                ConfigurationError::InvalidCloudProvider(format!(
                    "GCP GET {} failed: {}",
                    url, error
                ))
            })?;
        parse_response("GET", url, response).await
    }

    pub async fn post<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        eprintln!("[GCP] POST {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| {
                ConfigurationError::InvalidCloudProvider(format!(
                    "GCP POST {} failed: {}",
                    url, error
                ))
            })?;
        parse_response("POST", url, response).await
    }

    pub async fn patch<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        eprintln!("[GCP] PATCH {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .patch(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| {
                ConfigurationError::InvalidCloudProvider(format!(
                    "GCP PATCH {} failed: {}",
                    url, error
                ))
            })?;
        parse_response("PATCH", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        eprintln!("[GCP] DELETE {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http_client
            .delete(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| {
                ConfigurationError::InvalidCloudProvider(format!(
                    "GCP DELETE {} failed: {}",
                    url, error
                ))
            })?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "GCP DELETE {} returned {}: {}",
            url, status, body
        ))
        .into())
    }
}

async fn parse_response(method: &str, url: &str, response: Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().await.map_err(|error| {
        ConfigurationError::InvalidCloudProvider(format!(
            "Failed to read GCP response body: {}",
            error
        ))
    })?;

    if status.is_success() {
        eprintln!("[GCP] {} {} → {}", method, url, status);
        if body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&body).map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to parse GCP JSON: {} — body: {}",
                error, body
            ))
            .into()
        })
    } else {
        eprintln!("[GCP] {} {} → {} — {}", method, url, status, body);
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "GCP {} {} error {}: {}",
            method, url, status, body
        ))
        .into())
    }
}
