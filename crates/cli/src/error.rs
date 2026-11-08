use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] byocvpn_core::error::Error),

    #[error("No servers found")]
    NoInstancesFound,

    #[error("Selected entry is out of range")]
    SelectionOutOfRange,

    #[error("Prompt failed: {0}")]
    Prompt(#[from] dialoguer::Error),
}

pub type CliResult<T> = std::result::Result<T, CliError>;
