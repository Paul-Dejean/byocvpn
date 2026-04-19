use std::sync::Arc;

use azure_identity::ClientSecretCredential;
use byocvpn_core::error::{ComputeProvisioningError, Error, NetworkProvisioningError, Result};
use byocvpn_core::retry::retry;
use log::*;
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use tokio::time::Duration;

use crate::auth::get_access_token;
use crate::models::AsyncOperationResponse;

const ARM_BASE: &str = "https://management.azure.com";

const REQUEST_TIMEOUT_SECS: u64 = 90;

#[derive(Clone)]
pub struct AzureClient {
    http: HttpClient,
    credential: Arc<ClientSecretCredential>,
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

    pub fn build_arm_url(&self, path: &str, api_version: &str) -> String {
        format!("{}{}?api-version={}", ARM_BASE, path, api_version)
    }

    pub fn build_subscription_path(&self, rest: &str) -> String {
        format!("/subscriptions/{}{}", self.subscription_id, rest)
    }

    async fn get_access_token(&self) -> Result<String> {
        get_access_token(&self.credential).await
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        debug!("[Azure] GET {}", url);
        let token = self.get_access_token().await?;
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

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Option<String>> {
        debug!("[Azure] PUT {}", url);
        let token = self.get_access_token().await?;
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

    pub async fn delete(&self, url: &str) -> Result<Option<String>> {
        debug!("[Azure] DELETE {}", url);
        let token = self.get_access_token().await?;
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

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[Azure] POST {}", url);
        let token = self.get_access_token().await?;
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

    pub async fn wait_for_async_operation(&self, operation_url: &str) -> Result<()> {
        retry(
            || async move {
                let operation: AsyncOperationResponse = self.get(operation_url).await?;
                match operation.status.as_deref() {
                    Some("Succeeded") => Ok(()),
                    Some("Failed") | Some("Canceled") => {
                        let message = operation
                            .error
                            .as_ref()
                            .and_then(|error| error.message.as_deref())
                            .unwrap_or("unknown error");
                        Err(ComputeProvisioningError::InstanceSpawnFailed {
                            region_name: String::new(),
                            reason: format!("ARM operation failed: {}", message),
                        }
                        .into())
                    }
                    Some("InProgress") | Some("Running") | None => {
                        debug!("[Azure] ARM operation in progress...");
                        Err(Error::Transient {
                            operation_name: "ARM async operation".to_string(),
                        })
                    }
                    Some(other) => {
                        warn!("[Azure] Unexpected ARM operation status: {}", other);
                        Err(Error::Transient {
                            operation_name: "ARM async operation".to_string(),
                        })
                    }
                }
            },
            120,
            Duration::from_secs(5),
        )
        .await
    }
}

async fn parse_json_response<T: DeserializeOwned>(method: &str, url: &str, response: Response) -> Result<T> {
    let status = response.status();

    if status.is_success() {
        let body = response.text().await.map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure {} {} failed to read body: {}", method, url, error),
            }
        })?;
        let json_str = if body.is_empty() { "null" } else { body.as_str() };
        from_str(json_str).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!(
                    "Azure {} {} succeeded but body is not JSON: {} — body: {}",
                    method, url, error, body
                ),
            }
            .into()
        })
    } else if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        let error_body = response.text().await.unwrap_or_default();
        Err(Error::Authorization {
            operation: format!("Azure {} {}: {}", method, url, error_body),
        })
    } else {
        let error_body = response.text().await.unwrap_or_default();
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!(
                "Azure {} {} failed with status {}: {}",
                method, url, status, error_body
            ),
        }
        .into())
    }
}

async fn parse_lro_response(method: &str, url: &str, response: Response) -> Result<Option<String>> {
    let status = response.status();

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
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!(
                "Azure {} {} failed with status {}: {}",
                method, url, status, error_body
            ),
        }
        .into())
    }
}

pub fn extract_resource_group_from_id(resource_id: &str) -> Option<&str> {
    let segments: Vec<&str> = resource_id.split('/').collect();

    segments
        .windows(2)
        .find(|pair| pair[0].eq_ignore_ascii_case("resourceGroups"))
        .map(|pair| pair[1])
}

pub fn extract_name_from_id(resource_id: &str) -> Option<&str> {
    resource_id.split('/').filter(|s| !s.is_empty()).next_back()
}
