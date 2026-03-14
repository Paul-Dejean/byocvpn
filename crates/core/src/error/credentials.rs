use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("credentials file not found")]
    FileNotFound,

    #[error("failed to read credentials file: {reason}")]
    FileReadFailed { reason: String },

    #[error("failed to save credentials file: {reason}")]
    FileSaveFailed { reason: String },

    #[error("invalid credentials file format: {reason}")]
    InvalidFormat { reason: String },

    #[error("missing required credential: {field}")]
    MissingField { field: String },

    #[error("invalid private key: {reason}")]
    InvalidPrivateKey { reason: String },

    #[error("request signing failed: {reason}")]
    SigningFailed { reason: String },

    #[error("failed to acquire access token from {provider}: {reason}")]
    TokenAcquisitionFailed { provider: String, reason: String },

    #[error("no token returned by {provider}")]
    MissingTokenInResponse { provider: String },
}
