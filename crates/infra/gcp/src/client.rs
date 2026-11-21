use google_cloud_auth::credentials::Credentials;
use google_cloud_compute_v1::client::{
    Firewalls, Images, Instances, Networks, Regions, Subnetworks,
};
use log::debug;
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;

use crate::{
    auth::fetch_bearer_token,
    error::{ApiRequest, GcpError, Result},
};

const RESOURCE_LINK_BASE_URL: &str = "https://www.googleapis.com/compute/v1/projects";
const SERVICE_USAGE_BASE_URL: &str = "https://serviceusage.googleapis.com/v1";
const CLOUD_RESOURCE_MANAGER_BASE_URL: &str = "https://cloudresourcemanager.googleapis.com/v1";

#[derive(Debug)]
pub struct GcpClient {
    pub http: HttpClient,
    pub credentials: Credentials,
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
            .map_err(GcpError::client_build("instances"))?;
        let networks = Networks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(GcpError::client_build("networks"))?;
        let subnetworks = Subnetworks::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(GcpError::client_build("subnetworks"))?;
        let firewalls = Firewalls::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(GcpError::client_build("firewalls"))?;
        let regions = Regions::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(GcpError::client_build("regions"))?;
        let images = Images::builder()
            .with_credentials(credentials.clone())
            .build()
            .await
            .map_err(GcpError::client_build("images"))?;

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

    async fn send_request<B: Serialize + Sync>(
        &self,
        request: &ApiRequest,
        body: Option<&B>,
    ) -> Result<Response> {
        debug!("[GCP] {request}");
        let token = fetch_bearer_token(&self.credentials).await?;
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
    let body = response
        .text()
        .await
        .map_err(|source| request.transport(source))?;

    if !status.is_success() {
        debug!("[GCP] {request} -> {status}: {body}");
        return Err(request.rejected(status, &body));
    }

    debug!("[GCP] {request} -> {status}");
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|source| request.invalid_json(source))
}
