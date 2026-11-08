use azure_core::Error as AzureCoreError;
use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{Error as CoreError, ErrorKind},
};
use reqwest::{Error as ReqwestError, Method, StatusCode};
use serde::Deserialize;
use serde_json::Error as JsonError;
use strum::EnumString;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AzureError>;

const NOT_FOUND_STATUS: u16 = 404;

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub enum ArmErrorCode {
    MissingSubscriptionRegistration,
    SkuNotAvailable,
    ResourceNotFound,
    ResourceGroupNotFound,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Error)]
pub enum AzureError {
    #[error("{method} {url} rejected with HTTP {status} {}: {}", .code.as_deref().unwrap_or("no error code"), .message.as_deref().unwrap_or("no message"))]
    Rejected {
        method: String,
        url: String,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        body: String,
    },

    #[error("Azure request failed: {0}")]
    Transport(#[from] ReqwestError),

    #[error("{method} {url} returned a body that is not valid JSON: {source}")]
    InvalidJson {
        method: String,
        url: String,
        #[source]
        source: JsonError,
    },

    #[error("Azure credential error: {0}")]
    Credential(#[from] AzureCoreError),

    #[error("ARM operation failed: {message} ({operation_url})")]
    OperationFailed {
        operation_url: String,
        message: String,
    },

    #[error("ARM operation did not complete in time: {operation_url}")]
    OperationTimedOut { operation_url: String },

    #[error("resource provider {namespace} registration did not complete in time")]
    ProviderRegistrationTimedOut { namespace: String },

    #[error("no configured VM size is available in {location}")]
    NoVmSizeAvailable { location: String },

    #[error("instance identifier must be <resource group>/<vm name>")]
    InvalidInstanceId,

    #[error("no retail price for the {component} of {vm_size} in {region}")]
    PriceUnavailable {
        region: String,
        vm_size: String,
        component: &'static str,
    },

    #[error("retail prices API returned HTTP {status}")]
    RetailPricesRejected { status: u16 },

    #[error(transparent)]
    Core(#[from] CoreError),
}

impl AzureError {
    pub fn get_arm_error_code(&self) -> Option<ArmErrorCode> {
        match self {
            Self::Rejected {
                code: Some(code), ..
            } => code.parse().ok(),
            _ => None,
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Rejected { status, .. } if *status == NOT_FOUND_STATUS)
            || matches!(
                self.get_arm_error_code(),
                Some(ArmErrorCode::ResourceNotFound | ArmErrorCode::ResourceGroupNotFound)
            )
    }
}

impl From<AzureError> for CoreError {
    fn from(error: AzureError) -> Self {
        let kind = match error {
            AzureError::Rejected {
                method,
                url,
                status,
                code,
                message,
                body,
            } => {
                return Self::cloud_api_rejected(
                    CloudProviderName::Azure,
                    format!("{method} {url}"),
                    status,
                    code,
                    message,
                    &body,
                );
            }
            AzureError::Core(core_error) => return core_error,
            AzureError::Transport(_)
            | AzureError::OperationTimedOut { .. }
            | AzureError::ProviderRegistrationTimedOut { .. } => ErrorKind::Transient,
            AzureError::InvalidJson { .. } => ErrorKind::Unknown,
            AzureError::Credential(_) => ErrorKind::Authentication,
            AzureError::InvalidInstanceId => ErrorKind::Configuration,
            AzureError::OperationFailed { .. }
            | AzureError::NoVmSizeAvailable { .. }
            | AzureError::PriceUnavailable { .. }
            | AzureError::RetailPricesRejected { .. } => ErrorKind::Cloud,
        };
        Self::Provider {
            provider: CloudProviderName::Azure,
            kind,
            source: Box::new(error),
        }
    }
}

pub fn build_rejected_error(
    method: &Method,
    url: &str,
    status: StatusCode,
    body: String,
) -> AzureError {
    let arm_error = parse_arm_error(&body);
    AzureError::Rejected {
        method: method.to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        code: arm_error.as_ref().map(|arm_error| arm_error.code.clone()),
        message: arm_error.map(|arm_error| arm_error.message),
        body,
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
