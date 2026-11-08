use std::{io::Error as IoError, process::ExitStatus};

use serde_json::Error as JsonError;
use thiserror::Error;

use crate::cloud_provider::CloudProviderName;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("lock {lock_name} is poisoned")]
    MutexPoisoned { lock_name: &'static str },

    #[error("instance readiness probe failed: {reason}")]
    ReadinessProbeFailed { reason: String },

    #[error("instance did not become ready within the timeout period")]
    ReadinessProbeTimedOut,

    #[error("failed to {operation}: {source}")]
    TunnelIo {
        operation: &'static str,
        #[source]
        source: IoError,
    },

    #[error("kill switch command {command} failed to start: {source}")]
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

    #[error("kill switch failed to {operation} {path}: {source}")]
    KillSwitchFile {
        operation: &'static str,
        path: &'static str,
        #[source]
        source: IoError,
    },

    #[error("{provider} has no job step with id {step_id}")]
    UnknownJobStep {
        provider: CloudProviderName,
        step_id: String,
    },

    #[error("JSON serialization failed: {0}")]
    Json(#[from] JsonError),
}
