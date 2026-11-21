use byocvpn_core::error::Error as CoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("No servers found")]
    NoInstancesFound,

    #[error("Selected entry is out of range")]
    SelectionOutOfRange,

    #[error("Prompt failed: {0}")]
    Prompt(#[from] dialoguer::Error),
}

pub type CliResult<T> = Result<T, CliError>;
