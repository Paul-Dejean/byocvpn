use byocvpn_core::error::{NetworkProvisioningError, Result};
use google_cloud_auth::credentials::Credentials;
use log::{debug, error};
use reqwest::{Client as HttpClient, Method, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::{auth::fetch_bearer_token, error::build_request_error};

const COMPUTE_API_BASE_URL: &str = "https://compute.googleapis.com/compute/v1/projects";
const RESOURCE_LINK_BASE_URL: &str = "https://www.googleapis.com/compute/v1/projects";
const SERVICE_USAGE_BASE_URL: &str = "https://serviceusage.googleapis.com/v1";
const CLOUD_RESOURCE_MANAGER_BASE_URL: &str = "https://cloudresourcemanager.googleapis.com/v1";

pub struct GcpClient {
    http: HttpClient,
    credentials: Credentials,
    pub project_id: String,
}

impl GcpClient {
    pub fn new(credentials: Credentials, project_id: String) -> Self {
        Self {
            http: HttpClient::new(),
            credentials,
            project_id,
        }
    }

    pub fn build_compute_url(&self, path: &str) -> String {
        format!("{COMPUTE_API_BASE_URL}/{}/{path}", self.project_id)
    }

    pub fn build_resource_link(&self, path: &str) -> String {
        format!("{RESOURCE_LINK_BASE_URL}/{}/{path}", self.project_id)
    }

    pub fn build_service_usage_url(path: &str) -> String {
        format!("{SERVICE_USAGE_BASE_URL}/{path}")
    }

    pub fn build_project_url(&self, path: &str) -> String {
        format!(
            "{CLOUD_RESOURCE_MANAGER_BASE_URL}/projects/{}{path}",
            self.project_id
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

    pub async fn patch<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        let response = self.send_request(Method::PATCH, url, Some(body)).await?;
        parse_json_response("PATCH", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        let response = self.send_request(Method::DELETE, url, None::<&()>).await?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(build_request_error("DELETE", url, status, &body))
    }

    async fn send_request<B: Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<Response> {
        debug!("[GCP] {method} {url}");
        let token = fetch_bearer_token(&self.credentials).await?;
        let mut request = self.http.request(method.clone(), url).bearer_auth(&token);
        if let Some(body) = body {
            request = request.json(body);
        }
        request.send().await.map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("GCP {method} {url} failed: {error}"),
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
    let body =
        response
            .text()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to read GCP response body: {error}"),
            })?;

    if status == StatusCode::NOT_FOUND {
        return Err(NetworkProvisioningError::ResourceNotFound {
            url: url.to_string(),
        }
        .into());
    }
    if !status.is_success() {
        error!("[GCP] {method} {url} -> {status}: {body}");
        return Err(build_request_error(method, url, status, &body));
    }

    debug!("[GCP] {method} {url} -> {status}");
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|error| {
        NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("Failed to parse GCP JSON: {error} (body: {body})"),
        }
        .into()
    })
}
