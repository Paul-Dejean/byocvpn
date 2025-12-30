use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("daemon is not running")]
    NotRunning,

    #[error("failed to connect to daemon: {reason}")]
    ConnectionFailed { reason: String },

    #[error("daemon command failed: {command}")]
    CommandFailed { command: String },

    #[error("invalid daemon response: {reason}")]
    InvalidResponse { reason: String },

    #[error("daemon timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("daemon socket error: {reason}")]
    SocketError { reason: String },

    #[error("failed to start daemon: {reason}")]
    StartupFailed { reason: String },

    #[error("daemon already running")]
    AlreadyRunning,
}
