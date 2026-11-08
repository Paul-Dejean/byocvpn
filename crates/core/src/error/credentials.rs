use std::{io::Error as IoError, path::PathBuf};

use ini::ParseError as IniParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("no cloud provider credentials found; add your credentials in the desktop app first")]
    NoProviderConfigured,

    #[error("failed to {operation} the credentials directory {}: {source}", .path.display())]
    Directory {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("failed to read the credentials file: {0}")]
    FileRead(#[source] IoError),

    #[error("failed to save the credentials file: {0}")]
    FileSave(#[source] IoError),

    #[error("invalid credentials file format: {0}")]
    InvalidFormat(#[source] IniParseError),

    #[error("missing required credential: {field}")]
    MissingField { field: String },
}
