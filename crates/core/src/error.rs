use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Home directory not available")]
    HomeDirectoryNotAvailable,

    #[error("Invalid credentials file: {0}")]
    InvalidFile(String),

    #[error("Invalid cloud provider name: {0}")]
    InvalidCloudProviderName(String),
}

pub type Result<T> = std::result::Result<T, Error>;
