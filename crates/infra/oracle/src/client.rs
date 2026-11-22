use jiff::Timestamp;
use reqwest::{Client as HttpClient, Method, Response, StatusCode, Url};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_str, to_vec};
use tracing::{debug, instrument};

use crate::{
    auth::build_signed_headers,
    credentials::OracleCredentials,
    error::{ApiRequest, OracleError, Result},
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
        let request = ApiRequest::new(Method::GET, url);
        let response = self.send_request(&request, None).await?;
        parse_json_response(&request, response).await
    }

    pub async fn post<B: Serialize + Sync, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let request = ApiRequest::new(Method::POST, url);
        let body_bytes = to_vec(body).map_err(|source| request.invalid_body(source))?;
        let response = self.send_request(&request, Some(body_bytes)).await?;
        parse_json_response(&request, response).await
    }

    pub async fn put<B: Serialize + Sync>(&self, url: &str, body: &B) -> Result<()> {
        let request = ApiRequest::new(Method::PUT, url);
        let body_bytes = to_vec(body).map_err(|source| request.invalid_body(source))?;
        let response = self.send_request(&request, Some(body_bytes)).await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body_text = response
            .text()
            .await
            .map_err(|source| request.transport(source))?;
        debug!(%request, %status, body = %body_text, "request rejected");
        Err(request.rejected(status, &body_text))
    }

    pub async fn delete(&self, url: &str) -> Result<()> {
        let request = ApiRequest::new(Method::DELETE, url);
        let response = self.send_request(&request, None).await?;
        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let body_text = response
            .text()
            .await
            .map_err(|source| request.transport(source))?;
        Err(request.rejected(status, &body_text))
    }

    #[instrument(
        level = "debug",
        skip_all,
        fields(method = %request.get_method(), url = %request.get_url())
    )]
    async fn send_request(&self, request: &ApiRequest, body: Option<Vec<u8>>) -> Result<Response> {
        let url = request.get_url();
        let parsed_url = Url::parse(url).map_err(|source| OracleError::InvalidUrl {
            url: url.to_string(),
            source,
        })?;
        let host = parsed_url.host_str().unwrap_or_default();
        let path = parsed_url.query().map_or_else(
            || parsed_url.path().to_string(),
            |query| format!("{}?{query}", parsed_url.path()),
        );
        let date = Timestamp::now()
            .strftime("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();
        let signed_headers = build_signed_headers(
            &self.credentials,
            request.get_method(),
            host,
            &path,
            &date,
            body.as_deref(),
        )?;

        let mut http_request = self
            .http
            .request(request.get_method().clone(), url)
            .header("date", &date)
            .header("authorization", &signed_headers.authorization);
        if let (Some(body_bytes), Some(content_sha256)) = (body, signed_headers.content_sha256) {
            http_request = http_request
                .header("content-type", "application/json")
                .header("x-content-sha256", content_sha256)
                .header("content-length", body_bytes.len().to_string())
                .body(body_bytes);
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
        debug!(%request, %status, %body, "request rejected");
        return Err(request.rejected(status, &body));
    }

    debug!(%request, %status, "request succeeded");
    let json_text = if body.is_empty() { "null" } else { &body };
    from_str(json_text).map_err(|source| request.invalid_json(source))
}
