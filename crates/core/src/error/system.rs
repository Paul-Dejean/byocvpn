use thiserror::Error;

use crate::cloud_provider::CloudProviderName;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[error("instance readiness probe failed: {reason}")]
    ReadinessProbeFailed { reason: String },

    #[error("instance did not become ready within the timeout period")]
    ReadinessProbeTimedOut,

    #[error("tunnel I/O failed: {reason}")]
    TunnelIoFailed { reason: String },

    #[error("kill switch operation failed: {reason}")]
    KillSwitchFailed { reason: String },

    #[error("{provider} has no job step with id {step_id}")]
    UnknownJobStep {
        provider: CloudProviderName,
        step_id: String,
    },
}
