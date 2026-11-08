use std::sync::Arc;

use azure_identity::ClientSecretCredential;
use byocvpn_core::retry::{RetryPolicy, retry};
use log::{debug, warn};
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use tokio::time::Duration;

use crate::{
    arm_resource::ArmResource,
    auth::get_access_token,
    error::{AzureError, Result, build_rejected_error},
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
            .build()?;
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
        parse_json_response(&Method::GET, url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        let response = self.send_request(Method::POST, url, Some(body)).await?;
        parse_json_response(&Method::POST, url, response).await
    }

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Option<String>> {
        let response = self.send_request(Method::PUT, url, Some(body)).await?;
        parse_async_operation_response(&Method::PUT, url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<Option<String>> {
        let response = self.send_request(Method::DELETE, url, None::<&()>).await?;
        parse_async_operation_response(&Method::DELETE, url, response).await
    }

    pub async fn wait_for_async_operation(&self, operation_url: &str) -> Result<()> {
        retry(
            || async move {
                let operation: AsyncOperationResponse = self.get(operation_url).await?;
                match operation.status.as_deref() {
                    Some("Succeeded") => Ok(()),
                    Some("Failed" | "Canceled") => Err(AzureError::OperationFailed {
                        operation_url: operation_url.to_string(),
                        message: operation
                            .error
                            .and_then(|error| error.message)
                            .unwrap_or_else(|| "unknown error".to_string()),
                    }),
                    Some("InProgress" | "Running") | None => {
                        debug!("[Azure] ARM operation in progress...");
                        Err(AzureError::OperationTimedOut {
                            operation_url: operation_url.to_string(),
                        })
                    }
                    Some(other) => {
                        warn!("[Azure] Unexpected ARM operation status: {other}");
                        Err(AzureError::OperationTimedOut {
                            operation_url: operation_url.to_string(),
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
        let mut request = self.http.request(method, url).bearer_auth(&token);
        if let Some(body) = body {
            request = request.json(body);
        }
        Ok(request.send().await?)
    }
}

async fn parse_json_response<T: DeserializeOwned>(
    method: &Method,
    url: &str,
    response: Response,
) -> Result<T> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(build_rejected_error(method, url, status, error_body));
    }

    let body = response.text().await?;
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|source| AzureError::InvalidJson {
        method: method.to_string(),
        url: url.to_string(),
        source,
    })
}

async fn parse_async_operation_response(
    method: &Method,
    url: &str,
    response: Response,
) -> Result<Option<String>> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(build_rejected_error(method, url, status, error_body));
    }

    Ok(response
        .headers()
        .get("Azure-AsyncOperation")
        .or_else(|| response.headers().get("Location"))
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string))
}
