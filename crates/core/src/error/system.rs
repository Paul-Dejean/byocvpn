use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[error("instance readiness probe failed: {reason}")]
    ReadinessProbeFailed { reason: String },

    #[error("instance did not become ready within the timeout period")]
    ReadinessProbeTimedOut,
}
