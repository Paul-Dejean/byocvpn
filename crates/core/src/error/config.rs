use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("home directory not available")]
    HomeDirectoryNotAvailable,

    #[error("invalid configuration file: {reason}")]
    InvalidFile { reason: String },

    #[error("missing required field: {field}")]
    MissingField { field: String },

    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("failed to parse {value}: {reason}")]
    ParseError { value: String, reason: String },

    #[error("configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("tunnel configuration error: {reason}")]
    TunnelConfiguration { reason: String },

    #[error("route configuration error: {reason}")]
    RouteConfiguration { reason: String },

    #[error("template render error: {reason}")]
    TemplateRender { reason: String },

    #[error("invalid cloud provider: {0}")]
    InvalidCloudProvider(String),
}
