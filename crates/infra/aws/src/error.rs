use aws_sdk_ec2::{
    error::SdkError as Ec2SdkError,
    operation::{
        attach_internet_gateway::AttachInternetGatewayError,
        authorize_security_group_ingress::AuthorizeSecurityGroupIngressError,
        create_internet_gateway::CreateInternetGatewayError, create_route::CreateRouteError,
        create_security_group::CreateSecurityGroupError, create_subnet::CreateSubnetError,
        create_tags::CreateTagsError, create_vpc::CreateVpcError,
        describe_availability_zones::DescribeAvailabilityZonesError,
        describe_instances::DescribeInstancesError,
        describe_route_tables::DescribeRouteTablesError,
        describe_security_groups::DescribeSecurityGroupsError,
        describe_subnets::DescribeSubnetsError, describe_vpcs::DescribeVpcsError,
        modify_subnet_attribute::ModifySubnetAttributeError, run_instances::RunInstancesError,
        terminate_instances::TerminateInstancesError,
    },
};
use aws_sdk_ssm::{error::SdkError as SsmSdkError, operation::get_parameter::GetParameterError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not find SSM parameter: {0}")]
    MissingSsmParameter(String),

    #[error("SSM SDK error: {0}")]
    GetSsmParameterError(#[from] SsmSdkError<GetParameterError>),

    // Waiter errors (use string to avoid pinning the exact waiter error type)
    #[error("EC2 waiter 'instance running' failed: {0}")]
    Ec2WaitInstanceRunning(String),

    #[error("Security group '{0}' not found")]
    MissingSecurityGroup(String),

    #[error("RunInstances returned no instances")]
    NoInstanceInResponse,

    #[error("Instance is missing an instance_id")]
    InstanceMissingId,

    #[error("Instance has no public IPv4 yet")]
    MissingPublicIpv4,

    // If IPv6 is optional for you, prefer returning Option<String> instead of erroring.
    #[error("Instance has no public IPv6 yet")]
    MissingPublicIpv6,

    #[error("EC2 TerminateInstances failed: {0}")]
    Ec2TerminateInstances(#[from] Ec2SdkError<TerminateInstancesError>),

    #[error("No security group identifier returned by AWS")]
    MissingSecurityGroupIdentifier,

    #[error("No IPv6 CIDR block associated with VPC {vpc_identifier}")]
    MissingVpcIpv6Cidr { vpc_identifier: String },

    #[error("No main route table found for VPC {vpc_identifier}")]
    MissingMainRouteTable { vpc_identifier: String },

    // ---------- Parsing & validation ----------
    #[error("Invalid IPv6 CIDR `{cidr}`: {reason}")]
    InvalidIpv6Cidr { cidr: String, reason: String },

    // ---------- Standard conversions ----------
    #[error(transparent)]
    AddressParse(#[from] std::net::AddrParseError),

    #[error("EC2 CreateSecurityGroup failed: {0}")]
    Ec2CreateSecurityGroup(#[from] Ec2SdkError<CreateSecurityGroupError>),

    #[error("EC2 AuthorizeSecurityGroupIngress failed: {0}")]
    Ec2AuthorizeSecurityGroupIngress(#[from] Ec2SdkError<AuthorizeSecurityGroupIngressError>),

    #[error("EC2 DescribeSecurityGroups failed: {0}")]
    Ec2DescribeSecurityGroups(#[from] Ec2SdkError<DescribeSecurityGroupsError>),

    #[error("EC2 CreateVpc failed: {0}")]
    Ec2CreateVpc(#[from] Ec2SdkError<CreateVpcError>),

    #[error("EC2 DescribeVpcs failed: {0}")]
    Ec2DescribeVpcs(#[from] Ec2SdkError<DescribeVpcsError>),

    #[error("EC2 CreateSubnet failed: {0}")]
    Ec2CreateSubnet(#[from] Ec2SdkError<CreateSubnetError>),

    #[error("EC2 DescribeAvailabilityZones failed: {0}")]
    Ec2DescribeAvailabilityZones(#[from] Ec2SdkError<DescribeAvailabilityZonesError>),

    #[error("EC2 CreateInternetGateway failed: {0}")]
    Ec2CreateInternetGateway(#[from] Ec2SdkError<CreateInternetGatewayError>),

    #[error("EC2 AttachInternetGateway failed: {0}")]
    Ec2AttachInternetGateway(#[from] Ec2SdkError<AttachInternetGatewayError>),

    #[error("EC2 CreateRoute failed: {0}")]
    Ec2CreateRoute(#[from] Ec2SdkError<CreateRouteError>),

    #[error("EC2 ModifySubnetAttribute failed: {0}")]
    Ec2ModifySubnetAttribute(#[from] Ec2SdkError<ModifySubnetAttributeError>),

    #[error("EC2 DescribeRouteTables failed: {0}")]
    Ec2DescribeRouteTables(#[from] Ec2SdkError<DescribeRouteTablesError>),

    #[error("EC2 DescribeSubnets failed: {0}")]
    Ec2DescribeSubnets(#[from] Ec2SdkError<DescribeSubnetsError>),

    #[error("EC2 CreateTags failed: {0}")]
    Ec2CreateTags(#[from] Ec2SdkError<CreateTagsError>),

    #[error("EC2 RunInstances failed: {0}")]
    Ec2RunInstances(#[from] Ec2SdkError<RunInstancesError>),

    #[error("EC2 DescribeInstances failed: {0}")]
    Ec2DescribeInstances(#[from] Ec2SdkError<DescribeInstancesError>),
}

pub type Result<T> = std::result::Result<T, Error>;
