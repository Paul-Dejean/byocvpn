use std::fmt::{Display, Formatter, Result as FmtResult};

use azure_core::Error as AzureCoreError;
use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{CloudError, Error as CoreError, ErrorKind},
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
    #[error("{request} rejected with HTTP {status} {}: {}", .code.as_deref().unwrap_or("no error code"), .message.as_deref().unwrap_or(.body.as_str()))]
    Rejected {
        request: ApiRequest,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        body: String,
    },

    #[error("{request} could not be completed")]
    Transport {
        request: ApiRequest,
        #[source]
        source: ReqwestError,
    },

    #[error("failed to build the HTTP client")]
    HttpClientBuild(#[source] ReqwestError),

    #[error("{request} returned a body that is not valid JSON")]
    InvalidJson {
        request: ApiRequest,
        #[source]
        source: JsonError,
    },

    #[error("Azure authentication failed")]
    Authentication(#[from] AzureCoreError),

    #[error("ARM operation rejected: {message} ({operation_url})")]
    OperationRejected {
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

#[derive(Debug, Clone)]
pub struct ApiRequest {
    method: Method,
    url: String,
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

impl ApiRequest {
    pub fn new(method: Method, url: &str) -> Self {
        Self {
            method,
            url: url.to_string(),
        }
    }

    pub const fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn rejected(&self, status: StatusCode, body: String) -> AzureError {
        let arm_error = parse_arm_error(&body);
        AzureError::Rejected {
            request: self.clone(),
            status: status.as_u16(),
            code: arm_error.as_ref().map(|arm_error| arm_error.code.clone()),
            message: arm_error.map(|arm_error| arm_error.message),
            body,
        }
    }

    pub fn transport(&self, source: ReqwestError) -> AzureError {
        AzureError::Transport {
            request: self.clone(),
            source,
        }
    }

    pub fn invalid_json(&self, source: JsonError) -> AzureError {
        AzureError::InvalidJson {
            request: self.clone(),
            source,
        }
    }
}

impl CloudError for AzureError {
    fn provider(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    fn kind(&self) -> ErrorKind {
        match self {
            Self::Rejected { status, code, .. } => code
                .as_deref()
                .and_then(classify_arm_code)
                .unwrap_or_else(|| ErrorKind::from_http_status(*status)),
            Self::Transport { .. }
            | Self::OperationTimedOut { .. }
            | Self::ProviderRegistrationTimedOut { .. } => ErrorKind::Transient,
            Self::Authentication(_) => ErrorKind::Authentication,
            Self::InvalidInstanceId => ErrorKind::Configuration,
            Self::HttpClientBuild(_) | Self::InvalidJson { .. } | Self::Core(_) => {
                ErrorKind::Unknown
            }
            Self::OperationRejected { .. }
            | Self::NoVmSizeAvailable { .. }
            | Self::PriceUnavailable { .. }
            | Self::RetailPricesRejected { .. } => ErrorKind::Cloud,
        }
    }

    fn operation(&self) -> Option<String> {
        match self {
            Self::Rejected { request, .. }
            | Self::Transport { request, .. }
            | Self::InvalidJson { request, .. } => Some(request.to_string()),
            _ => None,
        }
    }

    fn code(&self) -> Option<String> {
        match self {
            Self::Rejected { code, .. } => code.clone(),
            _ => None,
        }
    }
}

impl From<AzureError> for CoreError {
    fn from(error: AzureError) -> Self {
        match error {
            AzureError::Core(core_error) => core_error,
            other => Self::Cloud(Box::new(other)),
        }
    }
}

impl Display for ApiRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{} {}", self.method, self.url)
    }
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

fn classify_arm_code(code: &str) -> Option<ErrorKind> {
    match code {
        "AuthorizationFailed" | "LinkedAuthorizationFailed" => Some(ErrorKind::Authorization),
        "InvalidAuthenticationToken" | "ExpiredAuthenticationToken" | "AuthenticationFailed" => {
            Some(ErrorKind::Authentication)
        }
        "TooManyRequests" | "QuotaExceeded" => Some(ErrorKind::Quota),
        "ResourceNotFound" | "ResourceGroupNotFound" | "NotFound" => Some(ErrorKind::NotFound),
        _ => None,
    }
}
