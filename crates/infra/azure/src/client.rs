use std::sync::Arc;

use azure_identity::ClientSecretCredential;
use byocvpn_core::{
    error::{ComputeProvisioningError, Error, NetworkProvisioningError, Result},
    retry::{RetryPolicy, retry},
};
use log::{debug, warn};
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use tokio::time::Duration;

use crate::{
    arm_resource::ArmResource, auth::get_access_token, error::build_request_error,
    models::AsyncOperationResponse,
};

const ARM_BASE_URL: &str = "https://management.azure.com";
const REQUEST_TIMEOUT_SECS: u64 = 90;
const ASYNC_OPERATION_POLL_POLICY: RetryPolicy = RetryPolicy::fixed(120, Duration::from_secs(5));

#[derive(Clone)]
pub struct AzureClient {
    http: HttpClient,
    credential: Arc<ClientSecretCredential>,
    pub subscription_id: String,
}

impl AzureClient {
    pub fn new(credential: Arc<ClientSecretCredential>, subscription_id: String) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|error| NetworkProvisioningError::CloudOperationFailed {
                reason: format!("failed to build HTTP client: {error}"),
            })?;
        Ok(Self {
            http,
            credential,
            subscription_id,
        })
    }

    pub fn build_arm_url(path: &str, api_version: &str) -> String {
        format!("{ARM_BASE_URL}{path}?api-version={api_version}")
    }

    pub fn build_subscription_path(&self, rest: &str) -> String {
        format!("/subscriptions/{}{}", self.subscription_id, rest)
    }

    pub fn build_resource_id(&self, resource: &ArmResource) -> String {
        self.build_subscription_path(resource.get_subscription_relative_path())
    }

    pub fn build_resource_url(&self, resource: &ArmResource) -> String {
        Self::build_arm_url(
            &self.build_resource_id(resource),
            resource.get_api_version(),
        )
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.send_request(Method::GET, url, None::<&()>).await?;
        parse_json_response("GET", url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        let response = self.send_request(Method::POST, url, Some(body)).await?;
        parse_json_response("POST", url, response).await
    }

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Option<String>> {
        let response = self.send_request(Method::PUT, url, Some(body)).await?;
        parse_async_operation_response("PUT", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<Option<String>> {
        let response = self.send_request(Method::DELETE, url, None::<&()>).await?;
        parse_async_operation_response("DELETE", url, response).await
    }

    pub async fn wait_for_async_operation(&self, operation_url: &str) -> Result<()> {
        retry(
            || async move {
                let operation: AsyncOperationResponse = self.get(operation_url).await?;
                match operation.status.as_deref() {
                    Some("Succeeded") => Ok(()),
                    Some("Failed" | "Canceled") => {
                        let message = operation
                            .error
                            .as_ref()
                            .and_then(|error| error.message.as_deref())
                            .unwrap_or("unknown error");
                        Err(ComputeProvisioningError::InstanceSpawnFailed {
                            region_name: String::new(),
                            reason: format!("ARM operation failed: {message}"),
                        }
                        .into())
                    }
                    Some("InProgress" | "Running") | None => {
                        debug!("[Azure] ARM operation in progress...");
                        Err(Error::Transient {
                            operation_name: "ARM async operation".to_string(),
                        })
                    }
                    Some(other) => {
                        warn!("[Azure] Unexpected ARM operation status: {other}");
                        Err(Error::Transient {
                            operation_name: "ARM async operation".to_string(),
                        })
                    }
                }
            },
            ASYNC_OPERATION_POLL_POLICY,
        )
        .await
    }

    async fn send_request<B: Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<Response> {
        debug!("[Azure] {method} {url}");
        let token = get_access_token(&self.credential).await?;
        let mut request = self.http.request(method.clone(), url).bearer_auth(&token);
        if let Some(body) = body {
            request = request.json(body);
        }
        request.send().await.map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure {method} {url} connection failed: {error}"),
            }
            .into()
        })
    }
}

async fn parse_json_response<T: DeserializeOwned>(
    method: &str,
    url: &str,
    response: Response,
) -> Result<T> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(build_request_error(method, url, status, &error_body));
    }

    let body =
        response
            .text()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Azure {method} {url} failed to read body: {error}"),
            })?;
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|error| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: format!(
                "Azure {method} {url} succeeded but body is not JSON: {error} (body: {body})"
            ),
        }
        .into()
    })
}

async fn parse_async_operation_response(
    method: &str,
    url: &str,
    response: Response,
) -> Result<Option<String>> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(build_request_error(method, url, status, &error_body));
    }

    Ok(response
        .headers()
        .get("Azure-AsyncOperation")
        .or_else(|| response.headers().get("Location"))
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string))
}
