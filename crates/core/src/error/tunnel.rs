use std::{io::Error as IoError, process::ExitStatus};

use ipnet::AddrParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("tunnel already running")]
    AlreadyRunning,

    #[error("failed to {operation} the TUN device")]
    TunDevice {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("failed to {operation}")]
    Io {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("route {operation} failed")]
    Route {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("invalid route destination {destination}")]
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

    #[error("{command} failed to start")]
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

    #[error("failed to update the NRPT DNS rule")]
    DnsRegistry(#[source] IoError),

    #[error("kill switch command {command} failed to start")]
    KillSwitchCommand {
        command: &'static str,
        #[source]
        source: IoError,
    },

    #[error("kill switch command {command} exited with {status}: {stderr}")]
    KillSwitchCommandRejected {
        command: &'static str,
        status: ExitStatus,
        stderr: String,
    },

    #[error("kill switch failed to {operation} {path}")]
    KillSwitchFile {
        operation: &'static str,
        path: &'static str,
        #[source]
        source: IoError,
    },
}

impl TunnelError {
    pub fn io(operation: &'static str) -> impl Fn(IoError) -> Self {
        move |source| Self::Io { operation, source }
    }

    pub fn route(operation: &'static str) -> impl Fn(IoError) -> Self {
        move |source| Self::Route { operation, source }
    }
}
