use std::{error::Error as StdError, io::Error as IoError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("daemon is not running")]
    NotRunning,

    #[error("daemon socket error: {0}")]
    Socket(#[from] IoError),

    #[error("daemon closed the connection without a response")]
    EmptyResponse,

    #[error("VPN service call failed: {0}")]
    Plugin(#[source] Box<dyn StdError + Send + Sync>),

    #[error("already connected to {instance_id}; disconnect first")]
    AlreadyConnected { instance_id: String },
}
