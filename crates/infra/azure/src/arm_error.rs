use byocvpn_core::error::Error;
use serde::Deserialize;
use strum::EnumString;

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub enum ArmErrorCode {
    MissingSubscriptionRegistration,
    SkuNotAvailable,
    #[strum(default)]
    Other(String),
}

pub struct ArmError {
    pub code: String,
    pub message: String,
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

pub fn extract_arm_error_code(error: &Error) -> Option<ArmErrorCode> {
    if let Error::CloudApiRejected { code, .. } = error {
        code.parse().ok()
    } else {
        None
    }
}

pub fn parse_arm_error(body: &str) -> Option<ArmError> {
    let response: ArmErrorResponse = serde_json::from_str(body).ok()?;
    let error_body = response.error?;
    let code = error_body.code.or_else(|| {
        error_body
            .details
            .first()
            .and_then(|detail| detail.code.clone())
    })?;
    let message = error_body.message.unwrap_or_default();
    Some(ArmError { code, message })
}
