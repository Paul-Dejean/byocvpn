use std::sync::Arc;

use azure_identity::ClientSecretCredential;
use byocvpn_core::retry::{RetryPolicy, retry};
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use tokio::time::Duration;
use tracing::{debug, instrument, warn};

use crate::{
    arm_resource::ArmResource,
    auth::get_access_token,
    error::{ApiRequest, AzureError, Result},
    models::AsyncOperationResponse,
    state::AzureOperationStatus,
};

const ARM_BASE_URL: &str = "https://management.azure.com";
const REQUEST_TIMEOUT_SECS: u64 = 90;
const ASYNC_OPERATION_POLL_POLICY: RetryPolicy = RetryPolicy::fixed(120, Duration::from_secs(5));

#[derive(Debug, Clone)]
pub struct AzureClient {
    http: HttpClient,
    credential: Arc<ClientSecretCredential>,
    subscription_id: String,
}

impl AzureClient {
    pub fn new(credential: Arc<ClientSecretCredential>, subscription_id: String) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(AzureError::HttpClientBuild)?;
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
        let request = ApiRequest::new(Method::GET, url);
        let response = self.send_request(&request, None::<&()>).await?;
        parse_json_response(&request, response).await
    }

    pub async fn post<B: Serialize + Sync, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let request = ApiRequest::new(Method::POST, url);
        let response = self.send_request(&request, Some(body)).await?;
        parse_json_response(&request, response).await
    }

    pub async fn put<B: Serialize + Sync>(&self, url: &str, body: &B) -> Result<Option<String>> {
        let request = ApiRequest::new(Method::PUT, url);
        let response = self.send_request(&request, Some(body)).await?;
        parse_async_operation_response(&request, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<Option<String>> {
        let request = ApiRequest::new(Method::DELETE, url);
        let response = self.send_request(&request, None::<&()>).await?;
        parse_async_operation_response(&request, response).await
    }

    pub async fn wait_for_async_operation(&self, operation_url: &str) -> Result<()> {
        retry(
            || async move {
                let operation: AsyncOperationResponse = self.get(operation_url).await?;
                match operation.status {
                    Some(AzureOperationStatus::Succeeded) => Ok(()),
                    Some(AzureOperationStatus::Failed | AzureOperationStatus::Canceled) => {
                        Err(AzureError::OperationRejected {
                            operation_url: operation_url.to_string(),
                            message: operation
                                .error
                                .and_then(|error| error.message)
                                .unwrap_or_else(|| "unknown error".to_string()),
                        })
                    }
                    Some(AzureOperationStatus::InProgress | AzureOperationStatus::Running)
                    | None => {
                        debug!("ARM operation in progress");
                        Err(AzureError::OperationTimedOut {
                            operation_url: operation_url.to_string(),
                        })
                    }
                    Some(AzureOperationStatus::Other(status)) => {
                        warn!(%status, "unexpected arm operation status");
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

    #[instrument(
        level = "debug",
        skip_all,
        fields(method = %request.get_method(), url = %request.get_url())
    )]
    async fn send_request<B: Serialize + Sync>(
        &self,
        request: &ApiRequest,
        body: Option<&B>,
    ) -> Result<Response> {
        let token = get_access_token(&self.credential).await?;
        let mut http_request = self
            .http
            .request(request.get_method().clone(), request.get_url())
            .bearer_auth(&token);
        if let Some(body) = body {
            http_request = http_request.json(body);
        }
        http_request
            .send()
            .await
            .map_err(|source| request.transport(source))
    }
}

async fn parse_json_response<T: DeserializeOwned>(
    request: &ApiRequest,
    response: Response,
) -> Result<T> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .map_err(|source| request.transport(source))?;
        return Err(request.rejected(status, error_body));
    }

    let body = response
        .text()
        .await
        .map_err(|source| request.transport(source))?;
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|source| request.invalid_json(source))
}

async fn parse_async_operation_response(
    request: &ApiRequest,
    response: Response,
) -> Result<Option<String>> {
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .map_err(|source| request.transport(source))?;
        return Err(request.rejected(status, error_body));
    }

    Ok(response
        .headers()
        .get("Azure-AsyncOperation")
        .or_else(|| response.headers().get("Location"))
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string))
}
