mod config;
mod credentials;
mod daemon;
mod instance;
mod internal;
mod tunnel;

use std::{error::Error as StdError, result::Result as StdResult};

pub use config::ConfigurationError;
pub use credentials::CredentialsError;
pub use daemon::DaemonError;
pub use instance::InstanceError;
pub use internal::InternalError;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Error as JsonError;
#[cfg(feature = "specta")]
use specta::{Type, Types, datatype::DataType};
use thiserror::Error;
pub use tunnel::TunnelError;

use crate::cloud_provider::CloudProviderName;

pub trait CloudError: StdError + Send + Sync + 'static {
    fn provider(&self) -> CloudProviderName;
    fn kind(&self) -> ErrorKind;
    fn operation(&self) -> Option<String> {
        None
    }
    fn code(&self) -> Option<String> {
        None
    }
}

impl StdError for Box<dyn CloudError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        (**self).source()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),

    #[error(transparent)]
    Tunnel(#[from] TunnelError),

    #[error(transparent)]
    Instance(#[from] InstanceError),

    #[error(transparent)]
    Internal(#[from] InternalError),

    #[error(transparent)]
    Credentials(#[from] CredentialsError),

    #[error(transparent)]
    Daemon(#[from] DaemonError),

    #[error("{}", .0.message)]
    Remote(ErrorPayload),

    #[error(transparent)]
    Cloud(Box<dyn CloudError>),
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
    Tunnel,
    Daemon,
    Cloud,
    Unknown,
}

impl ErrorKind {
    pub const fn from_http_status(status: u16) -> Self {
        match status {
            401 => Self::Authentication,
            403 => Self::Authorization,
            404 => Self::NotFound,
            429 => Self::Quota,
            500..=599 => Self::Transient,
            _ => Self::Cloud,
        }
    }
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

impl Error {
    pub fn describe(&self) -> String {
        describe_source_chain(self)
            .map_or_else(|| self.to_string(), |chain| format!("{self}: {chain}"))
    }

    pub fn is_not_found(&self) -> bool {
        self.kind() == ErrorKind::NotFound
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Credentials(_) | Self::Configuration(_) => ErrorKind::Configuration,
            Self::Daemon(_) => ErrorKind::Daemon,
            Self::Remote(payload) => payload.kind,
            Self::Cloud(error) => error.kind(),
            Self::Tunnel(_) => ErrorKind::Tunnel,
            Self::Instance(InstanceError::NotFound { .. }) => ErrorKind::NotFound,
            Self::Instance(InstanceError::ReadinessProbeTimedOut) => ErrorKind::Transient,
            Self::Instance(InstanceError::ReadinessProbeFailed { .. }) => ErrorKind::Cloud,
            Self::Instance(InstanceError::LaunchStepMissing { .. }) | Self::Internal(_) => {
                ErrorKind::Unknown
            }
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            Self::Remote(payload) => payload.clone(),
            Self::Cloud(error) => ErrorPayload {
                kind: error.kind(),
                message: error.to_string(),
                detail: describe_source_chain(error.as_ref()),
                operation: error.operation(),
                code: error.code(),
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

fn describe_source_chain(error: &dyn StdError) -> Option<String> {
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

impl From<JsonError> for Error {
    fn from(error: JsonError) -> Self {
        InternalError::Json(error).into()
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
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
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

pub type Result<T> = StdResult<T, Error>;
