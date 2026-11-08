use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComputeProvisioningError {
    #[error("instance {instance_id} was not found")]
    InstanceNotFound { instance_id: String },

    #[error("the launch step must run before {step}")]
    LaunchStepMissing { step: String },
}
