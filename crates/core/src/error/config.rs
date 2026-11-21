use std::{error::Error as StdError, io::Error as IoError, path::PathBuf};

use handlebars::RenderError;
use ini::Error as IniError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("home directory not available")]
    HomeDirectoryNotAvailable,

    #[error("failed to {operation} {}", .path.display())]
    DataFile {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("missing required field: {field}")]
    MissingField { field: String },

    #[error("invalid value for {field}: {reason}")]
    InvalidValue {
        field: &'static str,
        reason: &'static str,
    },

    #[error("failed to read the WireGuard config file")]
    WireguardConfigFile(#[source] IniError),

    #[error("invalid {field} in the WireGuard config")]
    InvalidWireguardField {
        field: &'static str,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("template render error")]
    TemplateRender(#[source] RenderError),

    #[error("unknown cloud provider name: {name}")]
    UnknownProviderName { name: String },
}
