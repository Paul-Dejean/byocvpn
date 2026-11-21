use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("instance {instance_id} was not found")]
    NotFound { instance_id: String },

    #[error("the launch step must run before {step}")]
    LaunchStepMissing { step: String },

    #[error("instance readiness probe failed: {reason}")]
    ReadinessProbeFailed { reason: String },

    #[error("instance did not become ready within the timeout period")]
    ReadinessProbeTimedOut,
}
