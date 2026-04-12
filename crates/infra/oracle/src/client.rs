use byocvpn_core::error::{NetworkProvisioningError, Result};
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::Serialize;
use serde_json::{Value, from_str, to_vec};

use crate::auth::{OciCredentials, build_authorization_header};
use log::*;

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

    pub fn build_core_base_url(&self) -> String {
        format!("https://iaas.{}.oraclecloud.com", self.credentials.region)
    }

    pub fn build_identity_base_url(&self) -> String {
        format!(
            "https://identity.{}.oraclecloud.com",
            self.credentials.region
        )
    }

    fn format_rfc7231_date() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let days_of_week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let mut remaining = secs;
        let seconds = remaining % 60;
        remaining /= 60;
        let minutes = remaining % 60;
        remaining /= 60;
        let hours = remaining % 24;
        remaining /= 24;

        let weekday = ((remaining + 4) % 7) as usize;

        let mut year = 1970u64;
        let mut day_of_year = remaining;
        loop {
            let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            };
            if day_of_year < days_in_year {
                break;
            }
            day_of_year -= days_in_year;
            year += 1;
        }
        let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 0usize;
        for (m, &days) in days_per_month.iter().enumerate() {
            if day_of_year < days {
                month = m;
                break;
            }
            day_of_year -= days;
        }
        let day = day_of_year + 1;
        format!(
            "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            days_of_week[weekday], day, months[month], year, hours, minutes, seconds
        )
    }

    pub async fn get(&self, url: &str) -> Result<Value> {
        debug!("[OCI] GET {}", url);
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {}: {}", url, error),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::format_rfc7231_date();

        let (authorization, _) =
            build_authorization_header("GET", &host, &path, &date, None, &self.credentials)?;

        let response = self
            .http_client
            .get(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI GET {} failed: {}", url, error),
            })?;

        parse_response("GET", url, response).await
    }

    pub async fn post<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        debug!("[OCI] POST {}", url);
        let body_bytes =
            to_vec(body).map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to serialize body: {}", error),
            })?;
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {}: {}", url, error),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::format_rfc7231_date();

        let (authorization, content_sha256) = build_authorization_header(
            "POST",
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
                reason: format!("OCI POST {} failed: {}", url, error),
            })?;

        parse_response("POST", url, response).await
    }

    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        debug!("[OCI] PUT {}", url);
        let body_bytes =
            to_vec(body).map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to serialize body: {}", error),
            })?;
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {}: {}", url, error),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::format_rfc7231_date();

        let (authorization, content_sha256) = build_authorization_header(
            "PUT",
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
                reason: format!("OCI PUT {} failed: {}", url, error),
            })?;

        parse_response("PUT", url, response).await
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        debug!("[OCI] DELETE {}", url);
        let parsed = reqwest::Url::parse(url).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Invalid URL {}: {}", url, error),
            }
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::format_rfc7231_date();

        let (authorization, _) =
            build_authorization_header("DELETE", &host, &path, &date, None, &self.credentials)?;

        let response = self
            .http_client
            .delete(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("OCI DELETE failed: {}", error),
            })?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("OCI DELETE {} returned {}: {}", url, status, body),
        }
        .into())
    }
}

async fn parse_response(method: &str, url: &str, response: Response) -> Result<Value> {
    let status = response.status();
    let body =
        response
            .text()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to read OCI response: {}", error),
            })?;

    if status.is_success() {
        debug!("[OCI] {} {} → {}", method, url, status);
        if body.is_empty() {
            return Ok(Value::Null);
        }
        from_str(&body).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: format!(
                    "Failed to parse OCI JSON response: {} — body: {}",
                    error, body
                ),
            }
            .into()
        })
    } else {
        error!("[OCI] {} {} → {} — {}", method, url, status, body);
        Err(NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("OCI {} {} error {}: {}", method, url, status, body),
        }
        .into())
    }
}
