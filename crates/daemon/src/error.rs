use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("http error: {0}")]
    Http(String),
    #[error("aws sdk error: {0}")]
    AwsSdk(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, InfrastructureError>;
