use byocvpn_core::error::{ConfigurationError, Result};
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::Serialize;
use serde_json::Value;

use crate::auth::{OciCredentials, build_authorization_header};

/// Thin HTTP client that automatically signs every request with OCI credentials.
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

    /// Returns the base URL for the OCI Core Services (Networking + Compute) API.
    pub fn core_base_url(&self) -> String {
        format!("https://iaas.{}.oraclecloud.com", self.credentials.region)
    }

    /// Returns the base URL for the OCI Identity API.
    pub fn identity_base_url(&self) -> String {
        format!(
            "https://identity.{}.oraclecloud.com",
            self.credentials.region
        )
    }

    fn rfc7231_date() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Build a simple RFC 7231 date from epoch (good enough for signing; OCI accepts ~5 min skew)
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
        // Days since epoch → weekday (0=Thu for Unix epoch Jan 1 1970)
        let weekday = ((remaining + 4) % 7) as usize;
        // Approximate year/month/day
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

    /// Perform a signed GET request to `url` and return the parsed JSON.
    pub async fn get(&self, url: &str) -> Result<Value> {
        eprintln!("[OCI] GET {}", url);
        let parsed = reqwest::Url::parse(url).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Invalid URL {}: {}", url, e))
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::rfc7231_date();

        let (authorization, _) =
            build_authorization_header("GET", &host, &path, &date, None, &self.credentials)?;

        let response = self
            .http_client
            .get(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|e| {
                ConfigurationError::InvalidCloudProvider(format!("OCI GET {} failed: {}", url, e))
            })?;

        parse_response("GET", url, response).await
    }

    /// Perform a signed POST request with a JSON body.
    pub async fn post<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        eprintln!("[OCI] POST {}", url);
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Failed to serialize body: {}", e))
        })?;
        let parsed = reqwest::Url::parse(url).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Invalid URL {}: {}", url, e))
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::rfc7231_date();

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
            .map_err(|e| {
                ConfigurationError::InvalidCloudProvider(format!("OCI POST {} failed: {}", url, e))
            })?;

        parse_response("POST", url, response).await
    }

    /// Perform a signed PUT request with a JSON body.
    pub async fn put<B: Serialize>(&self, url: &str, body: &B) -> Result<Value> {
        eprintln!("[OCI] PUT {}", url);
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Failed to serialize body: {}", e))
        })?;
        let parsed = reqwest::Url::parse(url).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Invalid URL {}: {}", url, e))
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::rfc7231_date();

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
            .map_err(|e| {
                ConfigurationError::InvalidCloudProvider(format!("OCI PUT {} failed: {}", url, e))
            })?;

        parse_response("PUT", url, response).await
    }

    /// Perform a signed DELETE request.
    pub async fn delete(&self, url: &str) -> Result<()> {
        eprintln!("[OCI] DELETE {}", url);
        let parsed = reqwest::Url::parse(url).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!("Invalid URL {}: {}", url, e))
        })?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        let path = match parsed.query() {
            Some(query) => format!("{}?{}", parsed.path(), query),
            None => parsed.path().to_string(),
        };
        let date = Self::rfc7231_date();

        let (authorization, _) =
            build_authorization_header("DELETE", &host, &path, &date, None, &self.credentials)?;

        let response = self
            .http_client
            .delete(url)
            .header("date", &date)
            .header("authorization", &authorization)
            .send()
            .await
            .map_err(|e| {
                ConfigurationError::InvalidCloudProvider(format!("OCI DELETE failed: {}", e))
            })?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "OCI DELETE {} returned {}: {}",
            url, status, body
        ))
        .into())
    }
}

async fn parse_response(method: &str, url: &str, response: Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().await.map_err(|e| {
        ConfigurationError::InvalidCloudProvider(format!("Failed to read OCI response: {}", e))
    })?;

    if status.is_success() {
        eprintln!("[OCI] {} {} → {}", method, url, status);
        if body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&body).map_err(|e| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to parse OCI JSON response: {} — body: {}",
                e, body
            ))
            .into()
        })
    } else {
        eprintln!("[OCI] {} {} → {} — {}", method, url, status, body);
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "OCI {} {} error {}: {}",
            method, url, status, body
        ))
        .into())
    }
}
