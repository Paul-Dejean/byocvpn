use byocvpn_core::{cloud_provider::CloudProviderName, error::Error};
use reqwest::StatusCode;
use serde::Deserialize;
use strum::EnumString;

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub enum ArmErrorCode {
    MissingSubscriptionRegistration,
    SkuNotAvailable,
    #[strum(default)]
    Other(String),
}

pub fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
    let arm_error = parse_arm_error(body);
    Error::cloud_api_rejected(
        CloudProviderName::Azure,
        format!("{method} {url}"),
        status.as_u16(),
        arm_error.as_ref().map(|arm_error| arm_error.code.clone()),
        arm_error.map(|arm_error| arm_error.message),
        body,
    )
}

pub fn extract_arm_error_code(error: &Error) -> Option<ArmErrorCode> {
    match error {
        Error::CloudApiRejected { code, .. } => code.parse().ok(),
        _ => None,
    }
}

struct ArmError {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct ArmErrorResponse {
    error: Option<ArmErrorBody>,
}

#[derive(Deserialize)]
struct ArmErrorBody {
    code: Option<String>,
    message: Option<String>,
    #[serde(default)]
    details: Vec<Self>,
}

fn parse_arm_error(body: &str) -> Option<ArmError> {
    let response: ArmErrorResponse = serde_json::from_str(body).ok()?;
    let error_body = response.error?;
    let code = error_body.code.or_else(|| {
        error_body
            .details
            .first()
            .and_then(|detail| detail.code.clone())
    })?;
    Some(ArmError {
        code,
        message: error_body.message.unwrap_or_default(),
    })
}
