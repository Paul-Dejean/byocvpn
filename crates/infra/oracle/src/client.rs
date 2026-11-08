use chrono::Utc;
use log::{debug, error};
use reqwest::{Client as HttpClient, Method, Response, StatusCode, Url};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_str, to_vec};

use crate::{
    auth::build_signed_headers,
    credentials::OracleCredentials,
    error::{Result, build_rejected_error},
};

const API_VERSION: &str = "20160918";

pub struct OciClient {
    http: HttpClient,
    credentials: OracleCredentials,
    region: String,
}

impl OciClient {
    pub fn new(credentials: OracleCredentials, region: String) -> Self {
        Self {
            http: HttpClient::new(),
            credentials,
            region,
        }
    }

    pub fn get_region(&self) -> &str {
        &self.region
    }

    pub fn get_compartment_ocid(&self) -> &str {
        &self.credentials.tenancy_ocid
    }

    pub fn build_core_url(&self, path: &str) -> String {
        format!(
            "https://iaas.{}.oraclecloud.com/{API_VERSION}{path}",
            self.region
        )
    }

    pub fn build_identity_url(&self, path: &str) -> String {
        format!(
            "https://identity.{}.oraclecloud.com/{API_VERSION}{path}",
            self.region
        )
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.send_request(Method::GET, url, None).await?;
        parse_json_response("GET", url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        let body_bytes = to_vec(body)?;
        let response = self
            .send_request(Method::POST, url, Some(body_bytes))
            .await?;
        parse_json_response("POST", url, response).await
    }

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<()> {
        let body_bytes = to_vec(body)?;
        let response = self
            .send_request(Method::PUT, url, Some(body_bytes))
            .await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body_text = response.text().await.unwrap_or_default();
        error!("[OCI] PUT {url} -> {status}: {body_text}");
        Err(build_rejected_error("PUT", url, status, &body_text))
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        let response = self.send_request(Method::DELETE, url, None).await?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body_text = response.text().await.unwrap_or_default();
        Err(build_rejected_error("DELETE", url, status, &body_text))
    }

    async fn send_request(
        &self,
        method: Method,
        url: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        debug!("[OCI] {method} {url}");
        let parsed_url = Url::parse(url)?;
        let host = parsed_url.host_str().unwrap_or_default();
        let path = match parsed_url.query() {
            Some(query) => format!("{}?{query}", parsed_url.path()),
            None => parsed_url.path().to_string(),
        };
        let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let signed_headers = build_signed_headers(
            &self.credentials,
            &method,
            host,
            &path,
            &date,
            body.as_deref(),
        )?;

        let mut request = self
            .http
            .request(method, url)
            .header("date", &date)
            .header("authorization", &signed_headers.authorization);
        if let (Some(body_bytes), Some(content_sha256)) = (body, signed_headers.content_sha256) {
            request = request
                .header("content-type", "application/json")
                .header("x-content-sha256", content_sha256)
                .header("content-length", body_bytes.len().to_string())
                .body(body_bytes);
        }
        Ok(request.send().await?)
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
        error!("[OCI] {method} {url} -> {status}: {body}");
        return Err(build_rejected_error(method, url, status, &body));
    }

    debug!("[OCI] {method} {url} -> {status}");
    let json_text = if body.is_empty() { "null" } else { &body };
    Ok(from_str(json_text)?)
}
