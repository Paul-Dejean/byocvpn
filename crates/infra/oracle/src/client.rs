use byocvpn_core::cloud_provider::CloudProviderName;
use byocvpn_core::error::{Error, NetworkProvisioningError, Result};
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{from_str, to_vec};

use crate::auth::{HttpMethod, OciCredentials, build_authorization_header};
use log::{debug, error};

pub struct OciClient {
    http_client: HttpClient,
    pub credentials: OciCredentials,
}

impl OciClient {
    pub fn new(credentials: OciCredentials) -> Self {
        Self {
            http_client: HttpClient::new(),
            credentials,
        }
    }

    pub fn build_core_url(&self, path: &str) -> String {
        format!(
            "https://iaas.{}.oraclecloud.com/20160918{}",
            self.credentials.region, path
        )
    }

    pub fn build_identity_url(&self, path: &str) -> String {
        format!(
            "https://identity.{}.oraclecloud.com/20160918{}",
            self.credentials.region, path
        )
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        debug!("[OCI] GET {url}");
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {url}: {error}"),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = format_rfc7231_date();

        let (authorization, _) = build_authorization_header(
            HttpMethod::Get,
            &host,
            &path,
            &date,
            None,
            &self.credentials,
        )?;

        let response = self
            .http_client
            .get(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI GET {url} failed: {error}"),
            })?;

        parse_response("GET", url, response).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, url: &str, body: &B) -> Result<T> {
        debug!("[OCI] POST {url}");
        let body_bytes =
            to_vec(body).map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to serialize body: {error}"),
            })?;
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {url}: {error}"),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = format_rfc7231_date();

        let (authorization, content_sha256) = build_authorization_header(
            HttpMethod::Post,
            &host,
            &path,
            &date,
            Some(&body_bytes),
            &self.credentials,
        )?;

        let response = self
            .http_client
            .post(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .header("content-type", "application/json")
            .header("x-content-sha256", &content_sha256)
            .header("content-length", body_bytes.len().to_string())
            .body(body_bytes)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI POST {url} failed: {error}"),
            })?;

        parse_response("POST", url, response).await
    }

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<()> {
        debug!("[OCI] PUT {url}");
        let body_bytes =
            to_vec(body).map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to serialize body: {error}"),
            })?;
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {url}: {error}"),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = format_rfc7231_date();

        let (authorization, content_sha256) = build_authorization_header(
            HttpMethod::Put,
            &host,
            &path,
            &date,
            Some(&body_bytes),
            &self.credentials,
        )?;

        let response = self
            .http_client
            .put(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .header("content-type", "application/json")
            .header("x-content-sha256", &content_sha256)
            .header("content-length", body_bytes.len().to_string())
            .body(body_bytes)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI PUT {url} failed: {error}"),
            })?;

        let status = response.status();
        if status.is_success() {
            debug!("[OCI] PUT {url} → {status}");
            return Ok(());
        }
        let body_text = response.text().await.unwrap_or_default();
        error!("[OCI] PUT {url} → {status} — {body_text}");
        Err(build_request_error("PUT", url, status, &body_text))
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("[OCI] DELETE {url}");
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {url}: {error}"),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = format_rfc7231_date();

        let (authorization, _) = build_authorization_header(
            HttpMethod::Delete,
            &host,
            &path,
            &date,
            None,
            &self.credentials,
        )?;

        let response = self
            .http_client
            .delete(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI DELETE failed: {error}"),
            })?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(build_request_error("DELETE", url, status, &body))
    }
}

fn format_rfc7231_date() -> String {
    chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string()
}

async fn parse_response<T: DeserializeOwned>(
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
                reason: format!("Failed to read OCI response: {error}"),
            })?;

    if status.is_success() {
        debug!("[OCI] {method} {url} → {status}");
        let json_str = if body.is_empty() {
            "null"
        } else {
            body.as_str()
        };
        from_str(json_str).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to parse OCI JSON response: {error} — body: {body}"),
            }
            .into()
        })
    } else {
        error!("[OCI] {method} {url} → {status} — {body}");
        Err(build_request_error(method, url, status, &body))
    }
}

#[derive(Deserialize)]
struct OciErrorResponse {
    code: Option<String>,
    message: Option<String>,
}

fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
    let error_body = from_str::<OciErrorResponse>(body).ok();
    Error::cloud_api_rejected(
        CloudProviderName::Oracle,
        format!("{method} {url}"),
        status.as_u16(),
        error_body
            .as_ref()
            .and_then(|error_body| error_body.code.clone()),
        error_body.and_then(|error_body| error_body.message),
        body,
    )
}
