use serde::Serialize;
use thiserror::Error;

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

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

    #[error("Invalid cloud provider config: {0}")]
    InvalidCloudProviderConfig(String),

    #[error("Tunnel creation error: {0}")]
    TunnelCreationError(String),
}

#[derive(Debug, Error)]
pub enum NetworkProvisioningError {
    #[error("network creation failed: {reason}")]
    NetworkCreationFailed { reason: String },

    #[error("subnet creation failed: {reason}")]
    SubnetCreationFailed { reason: String },

    #[error("security group creation failed: {reason}")]
    SecurityGroupCreationFailed { reason: String },

    #[error("security group not found: {group_name}")]
    SecurityGroupNotFound { group_name: String },

    #[error("security group rule configuration failed: {reason}")]
    SecurityGroupRuleConfigFailed { reason: String },

    #[error("internet gateway operation failed: {reason}")]
    InternetGatewayOperationFailed { reason: String },

    #[error("network query failed: {reason}")]
    NetworkQueryFailed { reason: String },

    #[error("subnet configuration failed: {reason}")]
    SubnetConfigurationFailed { reason: String },

    #[error("route table operation failed: {reason}")]
    RouteTableOperationFailed { reason: String },

    #[error("missing security group identifier in response")]
    MissingSecurityGroupIdentifier,

    #[error("missing VPC identifier in response")]
    MissingVpcIdentifier,

    #[error("missing subnet identifier in response")]
    MissingSubnetIdentifier,

    #[error("missing internet gateway identifier in response")]
    MissingInternetGatewayIdentifier,

    #[error("VPC missing IPv6 CIDR block: {vpc_id}")]
    MissingVpcIpv6Cidr { vpc_id: String },

    #[error("invalid IPv6 CIDR format: {cidr}")]
    InvalidIpv6Cidr { cidr: String },

    #[error("missing main route table for VPC: {vpc_id}")]
    MissingMainRouteTable { vpc_id: String },
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

    #[error("no instance returned in response")]
    NoInstanceInResponse,

    #[error("instance missing required identifier")]
    InstanceMissingId,

    #[error("failed to wait for instance to become running: {reason}")]
    InstanceWaitFailed { reason: String },

    #[error("instance missing public IPv4 address")]
    MissingPublicIpv4,

    #[error("instance missing public IPv6 address")]
    MissingPublicIpv6,
}

pub type Result<T> = std::result::Result<T, Error>;
