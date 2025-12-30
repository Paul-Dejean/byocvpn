use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkProvisioningError {
    #[error("virtual private cloud not found: {vpc_name}")]
    VpcNotFound { vpc_name: String },

    #[error("virtual private cloud creation failed: {reason}")]
    VpcCreationFailed { reason: String },

    #[error("subnet creation failed: {reason}")]
    SubnetCreationFailed { reason: String },

    #[error("subnet missing identifier")]
    SubnetMissingIdentifier,

    #[error("security group creation failed: {reason}")]
    SecurityGroupCreationFailed { reason: String },

    #[error("security group not found: {group_name}")]
    SecurityGroupNotFound { group_name: String },

    #[error("security group rule configuration failed: {reason}")]
    SecurityGroupRuleConfigurationFailed { reason: String },

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

    #[error("missing virtual private cloud identifier in response")]
    MissingVpcIdentifier,

    #[error("missing subnet identifier in response")]
    MissingSubnetIdentifier,

    #[error("missing internet gateway identifier in response")]
    MissingInternetGatewayIdentifier,

    #[error("virtual private cloud missing IPv6 CIDR block: {vpc_id}")]
    MissingVpcIpv6Cidr { vpc_id: String },

    #[error("invalid IPv6 CIDR format: {cidr}")]
    InvalidIpv6Cidr { cidr: String },

    #[error("missing main route table for virtual private cloud: {vpc_id}")]
    MissingMainRouteTable { vpc_id: String },
}
