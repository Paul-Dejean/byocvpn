use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComputeProvisioningError {
    #[error("instance spawn failed in region {region_name}: {reason}")]
    InstanceSpawnFailed { region_name: String, reason: String },

    #[error("instance termination failed for {instance_identifier}: {reason}")]
    InstanceTerminationFailed {
        instance_identifier: String,
        reason: String,
    },

    #[error("no instance returned in response")]
    NoInstanceInResponse,

    #[error("instance missing required identifier")]
    MissingInstanceIdentifier,

    #[error("failed to wait for instance to become running: {reason}")]
    InstanceWaitFailed { reason: String },

    #[error("instance missing public IPv4 address")]
    MissingPublicIpv4,

    #[error("instance missing public IPv6 address")]
    MissingPublicIpv6,

    #[error("AMI {name} lookup failed: {reason}")]
    AmiLookupFailed { name: String, reason: String },
}
