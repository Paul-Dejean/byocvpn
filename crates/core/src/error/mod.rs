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
use serde::Serialize;
pub use system::SystemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    InputOutput(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

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

    #[error("unexpected error during {operation_name}: {detail}")]
    Unknown {
        operation_name: String,
        detail: String,
    },
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
