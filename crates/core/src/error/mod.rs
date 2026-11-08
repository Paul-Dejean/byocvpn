mod compute;
mod config;
mod credentials;
mod daemon;
mod network;
mod system;

pub use compute::ComputeProvisioningError;
pub use config::ConfigurationError;
pub use credentials::CredentialsError;
pub use daemon::DaemonError;
pub use network::NetworkProvisioningError;
use serde::{Deserialize, Serialize};
#[cfg(feature = "specta")]
use specta::{Type, Types, datatype::DataType};
pub use system::SystemError;
use thiserror::Error;

use crate::cloud_provider::CloudProviderName;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] NetworkProvisioningError),

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

    #[error("authorization denied: {operation}")]
    Authorization { operation: String },

    #[error("authentication failed")]
    Authentication,

    #[error("rate limit or quota exceeded")]
    Quota,

    #[error("transient error during {operation_name}")]
    Transient { operation_name: String },

    #[error("{provider} rejected {operation} with HTTP {status} {code}: {message}")]
    CloudApiRejected {
        provider: CloudProviderName,
        operation: String,
        status: u16,
        code: String,
        message: String,
    },

    #[error("{operation}: {source}")]
    Context {
        operation: String,
        #[source]
        source: Box<Self>,
    },

    #[error("unexpected error during {operation_name}: {detail}")]
    Unknown {
        operation_name: String,
        detail: String,
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

    #[must_use]
    pub fn with_context(self, operation: impl Into<String>) -> Self {
        Self::Context {
            operation: operation.into(),
            source: Box::new(self),
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.kind() == ErrorKind::NotFound
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Authentication => ErrorKind::Authentication,
            Self::Authorization { .. } => ErrorKind::Authorization,
            Self::Quota => ErrorKind::Quota,
            Self::Transient { .. } => ErrorKind::Transient,
            Self::CloudApiRejected { status, .. } => classify_http_status(*status),
            Self::Context { source, .. } => source.kind(),
            Self::Credentials(error) => classify_credentials_error(error),
            Self::Configuration(_) => ErrorKind::Configuration,
            Self::Daemon(_) => ErrorKind::Daemon,
            Self::Network(error) => classify_network_error(error),
            Self::Compute(ComputeProvisioningError::InstanceNotFound { .. }) => ErrorKind::NotFound,
            Self::Compute(_) => ErrorKind::Cloud,
            Self::System(SystemError::ReadinessProbeTimedOut) => ErrorKind::Transient,
            Self::System(_) | Self::Unknown { .. } => ErrorKind::Unknown,
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            Self::Context { operation, source } => {
                let inner = source.to_payload();
                let inner_detail = inner.detail.unwrap_or_else(|| source.to_string());
                ErrorPayload {
                    kind: inner.kind,
                    message: inner.message,
                    detail: Some(format!("{operation}: {inner_detail}")),
                }
            }
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
            },
            _ => ErrorPayload {
                kind: self.kind(),
                message: self.to_string(),
                detail: None,
            },
        }
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

const fn classify_credentials_error(error: &CredentialsError) -> ErrorKind {
    match error {
        CredentialsError::InvalidPrivateKey { .. }
        | CredentialsError::SigningFailed { .. }
        | CredentialsError::TokenAcquisitionFailed { .. }
        | CredentialsError::MissingTokenInResponse { .. } => ErrorKind::Authentication,
        CredentialsError::FileNotFound
        | CredentialsError::NoProviderConfigured
        | CredentialsError::FileReadFailed { .. }
        | CredentialsError::FileSaveFailed { .. }
        | CredentialsError::InvalidFormat { .. }
        | CredentialsError::MissingField { .. } => ErrorKind::Configuration,
    }
}

const fn classify_network_error(error: &NetworkProvisioningError) -> ErrorKind {
    match error {
        NetworkProvisioningError::ResourceNotFound { .. } => ErrorKind::NotFound,
        NetworkProvisioningError::CloudOperationTimedOut { .. } => ErrorKind::Transient,
        _ => ErrorKind::Cloud,
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
