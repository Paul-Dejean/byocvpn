use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{Error as CoreError, ErrorKind},
};
use reqwest::{Error as ReqwestError, StatusCode};
use rsa::{pkcs8::Error as Pkcs8Error, signature::Error as SignatureError};
use serde::Deserialize;
use serde_json::{Error as JsonError, from_str};
use thiserror::Error;
use url::ParseError as UrlParseError;

pub type Result<T> = std::result::Result<T, OracleError>;

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("{method} {url} rejected with HTTP {status}: {}", .message.as_deref().unwrap_or("no message"))]
    Rejected {
        method: &'static str,
        url: String,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        body: String,
    },

    #[error("request to Oracle Cloud failed: {0}")]
    Transport(#[from] ReqwestError),

    #[error("invalid request URL: {0}")]
    InvalidUrl(#[from] UrlParseError),

    #[error("JSON serialization failed: {0}")]
    Json(#[from] JsonError),

    #[error("invalid private key: {0}")]
    InvalidPrivateKey(#[from] Pkcs8Error),

    #[error("request signing failed: {0}")]
    SigningFailed(#[from] SignatureError),

    #[error("{resource} response is missing {field}")]
    MissingResourceField {
        resource: &'static str,
        field: &'static str,
    },

    #[error("no availability domains found in {region}")]
    NoAvailabilityDomains { region: String },

    #[error("instance {instance_ocid} entered terminal state {lifecycle_state}")]
    InstanceTerminated {
        instance_ocid: String,
        lifecycle_state: String,
    },

    #[error("instance {instance_ocid} did not reach the running state in time")]
    InstanceNotRunning { instance_ocid: String },

    #[error("region {region} not found")]
    RegionNotFound { region: String },

    #[error("subscription to region {region} did not become ready in time")]
    RegionSubscriptionTimedOut { region: String },

    #[error("base image {image} not found")]
    BaseImageNotFound { image: &'static str },

    #[error("shape {shape} has no known part numbers")]
    UnknownShape { shape: String },

    #[error("price list returned HTTP {status} for part {part_number}")]
    PriceListRejected { part_number: String, status: u16 },

    #[error("no pay-as-you-go USD price found for part {part_number}")]
    PriceUnavailable { part_number: String },

    #[error(transparent)]
    Core(#[from] CoreError),
}

impl From<OracleError> for CoreError {
    fn from(error: OracleError) -> Self {
        let kind = match error {
            OracleError::Rejected {
                method,
                url,
                status,
                code,
                message,
                body,
            } => {
                return Self::cloud_api_rejected(
                    CloudProviderName::Oracle,
                    format!("{method} {url}"),
                    status,
                    code,
                    message,
                    &body,
                );
            }
            OracleError::Core(core_error) => return core_error,
            OracleError::Transport(_)
            | OracleError::InstanceNotRunning { .. }
            | OracleError::RegionSubscriptionTimedOut { .. } => ErrorKind::Transient,
            OracleError::InvalidPrivateKey(_) | OracleError::SigningFailed(_) => {
                ErrorKind::Authentication
            }
            OracleError::BaseImageNotFound { .. } | OracleError::RegionNotFound { .. } => {
                ErrorKind::NotFound
            }
            OracleError::InvalidUrl(_) | OracleError::Json(_) => ErrorKind::Unknown,
            OracleError::MissingResourceField { .. }
            | OracleError::NoAvailabilityDomains { .. }
            | OracleError::InstanceTerminated { .. }
            | OracleError::UnknownShape { .. }
            | OracleError::PriceListRejected { .. }
            | OracleError::PriceUnavailable { .. } => ErrorKind::Cloud,
        };
        Self::Provider {
            provider: CloudProviderName::Oracle,
            kind,
            source: Box::new(error),
        }
    }
}

pub fn build_rejected_error(
    method: &'static str,
    url: &str,
    status: StatusCode,
    body: &str,
) -> OracleError {
    let error_body = from_str::<OciErrorResponse>(body).ok();
    OracleError::Rejected {
        method,
        url: url.to_string(),
        status: status.as_u16(),
        code: error_body
            .as_ref()
            .and_then(|error_body| error_body.code.clone()),
        message: error_body.and_then(|error_body| error_body.message),
        body: body.to_string(),
    }
}

#[derive(Deserialize)]
struct OciErrorResponse {
    code: Option<String>,
    message: Option<String>,
}
