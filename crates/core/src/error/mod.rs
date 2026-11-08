mod compute;
mod config;
mod credentials;
mod daemon;
mod system;

pub use compute::ComputeProvisioningError;
pub use config::ConfigurationError;
pub use credentials::CredentialsError;
pub use daemon::DaemonError;
use std::error::Error as StdError;

use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
#[cfg(feature = "specta")]
use specta::{Type, Types, datatype::DataType};
pub use system::SystemError;
use thiserror::Error;

use crate::cloud_provider::CloudProviderName;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Compute(#[from] ComputeProvisioningError),

    #[error(transparent)]
    Configuration(#[from] ConfigurationError),

    #[error(transparent)]
    System(#[from] SystemError),

    #[error(transparent)]
    Credentials(#[from] CredentialsError),

    #[error(transparent)]
    Daemon(#[from] DaemonError),

    #[error("{}", .0.message)]
    Remote(ErrorPayload),

    #[error("{source}")]
    Provider {
        provider: CloudProviderName,
        kind: ErrorKind,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("{provider} rejected {operation} with HTTP {status} {code}: {message}")]
    CloudApiRejected {
        provider: CloudProviderName,
        operation: String,
        status: u16,
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ErrorKind {
    Authentication,
    Authorization,
    Quota,
    Transient,
    NotFound,
    Configuration,
    Daemon,
    Cloud,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ErrorPayload {
    pub kind: ErrorKind,
    pub message: String,
    pub detail: Option<String>,
    pub operation: Option<String>,
    pub code: Option<String>,
}

const RESPONSE_BODY_SUMMARY_LIMIT: usize = 300;

impl Error {
    pub fn cloud_api_rejected(
        provider: CloudProviderName,
        operation: impl Into<String>,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        raw_body: &str,
    ) -> Self {
        Self::CloudApiRejected {
            provider,
            operation: operation.into(),
            status,
            code: code.unwrap_or_else(|| format!("HTTP{status}")),
            message: message
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| summarize_response_body(raw_body)),
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.kind() == ErrorKind::NotFound
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::CloudApiRejected { status, code, .. } => {
                classify_cloud_api_code(code).unwrap_or_else(|| classify_http_status(*status))
            }
            Self::Credentials(_) => ErrorKind::Configuration,
            Self::Configuration(_) => ErrorKind::Configuration,
            Self::Daemon(_) => ErrorKind::Daemon,
            Self::Remote(payload) => payload.kind,
            Self::Provider { kind, .. } => *kind,
            Self::Compute(ComputeProvisioningError::InstanceNotFound { .. }) => ErrorKind::NotFound,
            Self::Compute(_) => ErrorKind::Cloud,
            Self::System(SystemError::ReadinessProbeTimedOut) => ErrorKind::Transient,
            Self::System(_) => ErrorKind::Unknown,
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            Self::Remote(payload) => payload.clone(),
            Self::CloudApiRejected {
                provider,
                operation,
                status,
                code,
                message,
            } => ErrorPayload {
                kind: self.kind(),
                message: message.clone(),
                detail: Some(format!(
                    "{provider} {operation} returned HTTP {status} {code}"
                )),
                operation: Some(operation.clone()),
                code: Some(code.clone()),
            },
            _ => ErrorPayload {
                kind: self.kind(),
                message: self.to_string(),
                detail: describe_source_chain(self),
                operation: None,
                code: None,
            },
        }
    }
}

fn describe_source_chain(error: &Error) -> Option<String> {
    let mut descriptions = Vec::new();
    let mut current = error.source();
    while let Some(source) = current {
        descriptions.push(source.to_string());
        current = source.source();
    }
    if descriptions.is_empty() {
        None
    } else {
        Some(descriptions.join(": "))
    }
}

fn summarize_response_body(raw_body: &str) -> String {
    let trimmed = raw_body.trim();
    if trimmed.is_empty() {
        return "empty response body".to_string();
    }
    let mut summary: String = trimmed.chars().take(RESPONSE_BODY_SUMMARY_LIMIT).collect();
    if summary.len() < trimmed.len() {
        summary.push('…');
    }
    summary
}

fn classify_cloud_api_code(code: &str) -> Option<ErrorKind> {
    let kind = match code {
        "UnauthorizedOperation"
        | "AccessDenied"
        | "AccessDeniedException"
        | "AuthorizationFailed"
        | "PERMISSION_DENIED"
        | "forbidden"
        | "NotAuthorizedOrNotFound" => ErrorKind::Authorization,
        "AuthFailure"
        | "InvalidClientTokenId"
        | "SignatureDoesNotMatch"
        | "UnrecognizedClientException"
        | "ExpiredToken"
        | "InvalidAuthenticationToken"
        | "ExpiredAuthenticationToken"
        | "UNAUTHENTICATED"
        | "authError"
        | "NotAuthenticated" => ErrorKind::Authentication,
        "ThrottlingException"
        | "RequestLimitExceeded"
        | "TooManyRequests"
        | "QuotaExceeded"
        | "RESOURCE_EXHAUSTED"
        | "QUOTA_EXCEEDED"
        | "rateLimitExceeded"
        | "quotaExceeded" => ErrorKind::Quota,
        "ResourceNotFound" | "NOT_FOUND" | "notFound" | "NotFound" => ErrorKind::NotFound,
        _ if code.ends_with(".NotFound") => ErrorKind::NotFound,
        _ => return None,
    };
    Some(kind)
}

const fn classify_http_status(status: u16) -> ErrorKind {
    match status {
        401 => ErrorKind::Authentication,
        403 => ErrorKind::Authorization,
        404 => ErrorKind::NotFound,
        429 => ErrorKind::Quota,
        500..=599 => ErrorKind::Transient,
        _ => ErrorKind::Cloud,
    }
}

impl From<JsonError> for Error {
    fn from(error: JsonError) -> Self {
        SystemError::Json(error).into()
    }
}

impl From<strum::ParseError> for Error {
    fn from(error: strum::ParseError) -> Self {
        ConfigurationError::UnknownProviderName {
            name: error.to_string(),
        }
        .into()
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}

#[cfg(feature = "specta")]
impl Type for Error {
    fn definition(types: &mut Types) -> DataType {
        <ErrorPayload as Type>::definition(types)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
