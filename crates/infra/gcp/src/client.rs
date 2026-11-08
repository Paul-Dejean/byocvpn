use byocvpn_core::error::{NetworkProvisioningError, Result};
use google_cloud_auth::credentials::Credentials;
use google_cloud_compute_v1::client::{
    Firewalls, Images, Instances, Networks, Regions, Subnetworks,
};
use google_cloud_gax::client_builder::Error as ClientBuilderError;
use log::{debug, error};
use reqwest::{Client as HttpClient, Method, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::{auth::fetch_bearer_token, error::build_request_error};

const RESOURCE_LINK_BASE_URL: &str = "https://www.googleapis.com/compute/v1/projects";
const SERVICE_USAGE_BASE_URL: &str = "https://serviceusage.googleapis.com/v1";
const CLOUD_RESOURCE_MANAGER_BASE_URL: &str = "https://cloudresourcemanager.googleapis.com/v1";

pub struct GcpClient {
    http: HttpClient,
    credentials: Credentials,
    pub project_id: String,
    pub instances: Instances,
    pub networks: Networks,
    pub subnetworks: Subnetworks,
    pub firewalls: Firewalls,
    pub regions: Regions,
    pub images: Images,
}

impl GcpClient {
    pub async fn connect(credentials: Credentials, project_id: String) -> Result<Self> {
        let instances = Instances::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("instances", error))?;
        let networks = Networks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("networks", error))?;
        let subnetworks = Subnetworks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("subnetworks", error))?;
        let firewalls = Firewalls::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("firewalls", error))?;
        let regions = Regions::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("regions", error))?;
        let images = Images::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|error| build_client_error("images", error))?;

        Ok(Self {
            http: HttpClient::new(),
            credentials,
            project_id,
            instances,
            networks,
            subnetworks,
            firewalls,
            regions,
            images,
        })
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

fn build_client_error(client_name: &str, error: ClientBuilderError) -> byocvpn_core::error::Error {
    NetworkProvisioningError::NetworkQueryFailed {
        reason: format!("failed to build GCP {client_name} client: {error}"),
    }
    .into()
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
