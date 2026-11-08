use byocvpn_core::{cloud_provider::CloudProviderName, error::Error};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize)]
struct OciErrorResponse {
    code: Option<String>,
    message: Option<String>,
}

pub fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
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
