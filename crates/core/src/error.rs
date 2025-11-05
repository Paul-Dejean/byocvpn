use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Home directory not available")]
    HomeDirectoryNotAvailable,

    #[error("Invalid credentials file: {0}")]
    InvalidCredentialsFile(String),

    #[error("Invalid cloud provider name: {0}")]
    InvalidCloudProviderName(String),

    #[error("base image not found {0}")]
    BaseImageNotFound(String),

    #[error(transparent)]
    Network(#[from] NetworkProvisioningError),

    #[error("rate limit or quota exceeded")]
    Quota,

    #[error(transparent)]
    Compute(#[from] ComputeProvisioningError),

    #[error("authorization error: {0}")]
    Authorization(String),

    #[error("unexpected failure during {operation_name}: {detail}")]
    Unknown {
        operation_name: &'static str,
        detail: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("temporary infrastructure failure during {operation_name}")]
    Transient {
        operation_name: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, Error)]
pub enum NetworkProvisioningError {
    #[error("virtual private cloud not found: {vpc_name}")]
    VirtualPrivateCloudNotFound { vpc_name: String },
}

#[derive(Debug, Error)]
pub enum ComputeProvisioningError {
    #[error("instance spawn failed in region {region_name}: {reason}")]
    InstanceSpawnFailed { region_name: String, reason: String },

    #[error("instance termination failed for {instance_identifier}: {reason}")]
    InstanceTerminationFailed {
        instance_identifier: String,
        reason: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
