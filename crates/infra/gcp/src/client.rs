use google_cloud_auth::credentials::Credentials;
use google_cloud_compute_v1::client::{
    Firewalls, Images, Instances, Networks, Regions, Subnetworks,
};
use google_cloud_gax::client_builder::Error as ClientBuilderError;
use log::{debug, error};
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::{
    auth::fetch_bearer_token,
    error::{GcpError, Result, build_rejected_error},
};

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
            .map_err(|source| build_client_error("instances", source))?;
        let networks = Networks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|source| build_client_error("networks", source))?;
        let subnetworks = Subnetworks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|source| build_client_error("subnetworks", source))?;
        let firewalls = Firewalls::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|source| build_client_error("firewalls", source))?;
        let regions = Regions::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|source| build_client_error("regions", source))?;
        let images = Images::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(|source| build_client_error("images", source))?;

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
        let mut request = self.http.request(method, url).bearer_auth(&token);
        if let Some(body) = body {
            request = request.json(body);
        }
        Ok(request.send().await?)
    }
}

const fn build_client_error(client_name: &'static str, source: ClientBuilderError) -> GcpError {
    GcpError::ClientBuild {
        client_name,
        source,
    }
}

async fn parse_json_response<T: DeserializeOwned>(
    method: &'static str,
    url: &str,
    response: Response,
) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        error!("[GCP] {method} {url} -> {status}: {body}");
        return Err(build_rejected_error(method, url, status.as_u16(), &body));
    }

    debug!("[GCP] {method} {url} -> {status}");
    let json_text = if body.is_empty() { "null" } else { &body };
    Ok(from_str(json_text)?)
}
