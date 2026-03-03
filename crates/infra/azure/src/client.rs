use std::sync::Arc;

/// Thin HTTP client for the Azure Resource Manager (ARM) REST API.
///
/// Authenticates every request with a Bearer token obtained from the
/// `azure_identity::ClientSecretCredential`.  All resource creation /
/// update operations that may return a `201 Created` or `202 Accepted`
/// response are handled by polling the embedded async-operation URL until
/// the operation reaches a terminal state (`Succeeded` / `Failed` /
/// `Canceled`).
use azure_identity::ClientSecretCredential;
use byocvpn_core::error::{
    ComputeProvisioningError, ConfigurationError, Error, NetworkProvisioningError, Result,
};
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::Serialize;
use serde_json::Value;
use tokio::time::{Duration, sleep};

use crate::auth::bearer_token;

const ARM_BASE: &str = "https://management.azure.com";
/// Timeout applied to every individual ARM HTTP request (connect + response headers + body).
const REQUEST_TIMEOUT_SECS: u64 = 90;

/// ARM HTTP client authenticated with a service-principal credential.
pub struct AzureClient {
    http: HttpClient,
    credential: Arc<ClientSecretCredential>,
    /// Azure subscription ID.
    pub subscription_id: String,
}

impl AzureClient {
    pub fn new(credential: Arc<ClientSecretCredential>, subscription_id: String) -> Self {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("failed to build reqwest client");
        Self {
            http,
            credential,
            subscription_id,
        }
    }

    // ---------------------------------------------------------------------------
    // URL helpers
    // ---------------------------------------------------------------------------

    /// Build a fully-qualified ARM URL: `https://management.azure.com{path}?api-version={version}`.
    pub fn arm_url(&self, path: &str, api_version: &str) -> String {
        format!("{}{}?api-version={}", ARM_BASE, path, api_version)
    }

    /// Return the subscription-scoped ARM path prefix.
    ///
    /// ```text
    /// /subscriptions/{subscription_id}{rest}
    /// ```
    pub fn subscription_path(&self, rest: &str) -> String {
        format!("/subscriptions/{}{}", self.subscription_id, rest)
    }

    // ---------------------------------------------------------------------------
    // Token
    // ---------------------------------------------------------------------------

    async fn get_bearer_token(&self) -> Result<String> {
        bearer_token(&self.credential).await
    }

    // ---------------------------------------------------------------------------
    // HTTP verbs
    // ---------------------------------------------------------------------------

    /// Perform an authenticated GET and return the JSON response body.
    ///
    /// Use this for both resource GETs (append `?api-version=` yourself) and
    /// for polling async-operation URLs (which already contain query params).
    pub async fn get(&self, url: &str) -> Result<Value> {
        eprintln!("[Azure] GET {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure GET {} connection failed: {}", url, error),
            })?;
        parse_json_response("GET", url, response).await
    }

    /// Perform an authenticated PUT.
    ///
    /// Returns the `Azure-AsyncOperation` or `Location` header value if the
    /// server responded with `201 Created` / `202 Accepted`, indicating a
    /// long-running operation.  Callers should pass this URL to
    /// [`wait_for_async_operation`].
    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Option<String>> {
        eprintln!("[Azure] PUT {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http
            .put(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure PUT {} connection failed: {}", url, error),
            })?;
        parse_lro_response("PUT", url, response).await
    }

    /// Perform an authenticated DELETE.
    ///
    /// Returns the async-operation URL if present (`202 Accepted`).
    pub async fn delete(&self, url: &str) -> Result<Option<String>> {
        eprintln!("[Azure] DELETE {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http
            .delete(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure DELETE {} connection failed: {}", url, error),
            })?;
        parse_lro_response("DELETE", url, response).await
    }

    /// Perform an authenticated POST and return the JSON response body.
    pub async fn post<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        eprintln!("[Azure] POST {}", url);
        let token = self.get_bearer_token().await?;
        let response = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(body)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure POST {} connection failed: {}", url, error),
            })?;
        parse_json_response("POST", url, response).await
    }

    // ---------------------------------------------------------------------------
    // Long-running operation (LRO) polling
    // ---------------------------------------------------------------------------

    /// Poll an ARM async-operation URL until it reaches a terminal state.
    ///
    /// Azure returns `{"status": "InProgress"}` while the operation is
    /// running; `{"status": "Succeeded"}` on success; and
    /// `{"status": "Failed"}` or `{"status": "Canceled"}` on failure.
    pub async fn wait_for_async_operation(&self, operation_url: &str) -> Result<()> {
        for attempt in 1..=120u32 {
            let body = self.get(operation_url).await?;
            match body["status"].as_str() {
                Some("Succeeded") => return Ok(()),
                Some("Failed") | Some("Canceled") => {
                    let message = body["error"]["message"].as_str().unwrap_or("unknown error");
                    return Err(ComputeProvisioningError::InstanceSpawnFailed {
                        region_name: String::new(),
                        reason: format!("ARM operation failed: {}", message),
                    }
                    .into());
                }
                Some("InProgress") | Some("Running") | None => {
                    eprintln!(
                        "[Azure] ARM operation in progress (attempt {}/120)...",
                        attempt
                    );
                    sleep(Duration::from_secs(5)).await;
                }
                Some(other) => {
                    eprintln!("[Azure] Unexpected ARM operation status: {}", other);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        Err(Error::Transient {
            operation_name: "ARM async operation".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Response parsing helpers
// ---------------------------------------------------------------------------

/// Parse a successful response body as JSON, or surface the error body.
async fn parse_json_response(method: &str, url: &str, response: Response) -> Result<Value> {
    let status = response.status();

    if status.is_success() {
        if status == StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }
        response.json::<Value>().await.map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Azure {} {} succeeded but body is not JSON: {}",
                method, url, error
            ))
            .into()
        })
    } else if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        let error_body = response.text().await.unwrap_or_default();
        Err(Error::Authorization {
            operation: format!("Azure {} {}: {}", method, url, error_body),
        })
    } else {
        let error_body = response.text().await.unwrap_or_default();
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "Azure {} {} failed with status {}: {}",
            method, url, status, error_body
        ))
        .into())
    }
}

/// Extract the async-operation URL from a PUT/DELETE response, or surface errors.
async fn parse_lro_response(method: &str, url: &str, response: Response) -> Result<Option<String>> {
    let status = response.status();

    // Prefer `Azure-AsyncOperation` over `Location` for polling.
    let async_op_url = response
        .headers()
        .get("Azure-AsyncOperation")
        .or_else(|| response.headers().get("Location"))
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string());

    if status.is_success() {
        Ok(async_op_url)
    } else if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        let error_body = response.text().await.unwrap_or_default();
        Err(Error::Authorization {
            operation: format!("Azure {} {}: {}", method, url, error_body),
        })
    } else {
        let error_body = response.text().await.unwrap_or_default();
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "Azure {} {} failed with status {}: {}",
            method, url, status, error_body
        ))
        .into())
    }
}

// ---------------------------------------------------------------------------
// Helpers for callers
// ---------------------------------------------------------------------------

/// Parse the resource group name out of an ARM resource ID.
///
/// ARM IDs follow the pattern:
/// `/subscriptions/{sub}/resourceGroups/{rg}/providers/{ns}/{type}/{name}`
pub fn resource_group_from_id(resource_id: &str) -> Option<&str> {
    let segments: Vec<&str> = resource_id.split('/').collect();
    // Find the index of "resourceGroups" and return the next segment.
    segments
        .windows(2)
        .find(|pair| pair[0].eq_ignore_ascii_case("resourceGroups"))
        .map(|pair| pair[1])
}

/// Parse the resource name (last path segment) from an ARM resource ID.
pub fn name_from_id(resource_id: &str) -> Option<&str> {
    resource_id.split('/').filter(|s| !s.is_empty()).last()
}
