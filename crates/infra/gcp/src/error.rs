use byocvpn_core::{cloud_provider::CloudProviderName, error::Error};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize)]
struct GcpErrorResponse {
    error: Option<GcpErrorBody>,
}

#[derive(Deserialize)]
struct GcpErrorBody {
    message: Option<String>,
    status: Option<String>,
    #[serde(default)]
    errors: Vec<GcpErrorDetail>,
}

#[derive(Deserialize)]
struct GcpErrorDetail {
    reason: Option<String>,
}

pub fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
    let error_body = from_str::<GcpErrorResponse>(body)
        .ok()
        .and_then(|response| response.error);
    let code = error_body.as_ref().and_then(|error_body| {
        error_body.status.clone().or_else(|| {
            error_body
                .errors
                .first()
                .and_then(|detail| detail.reason.clone())
        })
    });
    let message = error_body.and_then(|error_body| error_body.message);
    Error::cloud_api_rejected(
        CloudProviderName::Gcp,
        format!("{method} {url}"),
        status.as_u16(),
        code,
        message,
        body,
    )
}
