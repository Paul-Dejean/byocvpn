use std::{error::Error as StdError, io::Error as IoError, path::PathBuf, process::ExitStatus};

use handlebars::RenderError;
use ini::Error as IniError;
use ipnet::AddrParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("home directory not available")]
    HomeDirectoryNotAvailable,

    #[error("failed to {operation} {}: {source}", .path.display())]
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

    #[error("failed to read the WireGuard config file: {0}")]
    WireguardConfigFile(#[source] IniError),

    #[error("invalid {field} in the WireGuard config: {source}")]
    InvalidWireguardField {
        field: &'static str,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("template render error: {0}")]
    TemplateRender(#[source] RenderError),

    #[error("tunnel already running")]
    TunnelAlreadyRunning,

    #[error("failed to {operation} the TUN device: {source}")]
    TunDevice {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("route {operation} failed: {source}")]
    Route {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("invalid route destination {destination}: {source}")]
    InvalidRouteDestination {
        destination: String,
        #[source]
        source: AddrParseError,
    },

    #[error("no default route found")]
    NoDefaultRoute,

    #[error("default route has no gateway")]
    DefaultRouteWithoutGateway,

    #[error("DNS override requested with no servers")]
    NoDnsServers,

    #[error("{command} failed to start: {source}")]
    DnsCommand {
        command: &'static str,
        #[source]
        source: IoError,
    },

    #[error("{command} exited with {status}: {stderr}")]
    DnsCommandRejected {
        command: &'static str,
        status: ExitStatus,
        stderr: String,
    },

    #[error("failed to update the NRPT DNS rule: {0}")]
    DnsRegistry(#[source] IoError),

    #[error("unknown cloud provider name: {name}")]
    UnknownProviderName { name: String },
}
