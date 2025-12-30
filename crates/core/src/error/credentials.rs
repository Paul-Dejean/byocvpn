use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("credentials file not found")]
    FileNotFound,

    #[error("invalid credentials file format: {reason}")]
    InvalidFormat { reason: String },

    #[error("missing required credential: {field}")]
    MissingField { field: String },
}
