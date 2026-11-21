use std::error::Error as StdError;

use aws_sdk_ec2::{
    error::{BuildError, DisplayErrorContext, ProvideErrorMetadata, SdkError},
    operation::{
        attach_internet_gateway::AttachInternetGatewayError,
        authorize_security_group_ingress::AuthorizeSecurityGroupIngressError,
        create_internet_gateway::CreateInternetGatewayError, create_route::CreateRouteError,
        create_security_group::CreateSecurityGroupError, create_subnet::CreateSubnetError,
        create_tags::CreateTagsError, create_vpc::CreateVpcError,
        describe_availability_zones::DescribeAvailabilityZonesError,
        describe_instances::DescribeInstancesError,
        describe_internet_gateways::DescribeInternetGatewaysError,
        describe_regions::DescribeRegionsError, describe_route_tables::DescribeRouteTablesError,
        describe_security_groups::DescribeSecurityGroupsError,
        describe_subnets::DescribeSubnetsError, describe_vpcs::DescribeVpcsError,
        modify_subnet_attribute::ModifySubnetAttributeError,
        revoke_security_group_ingress::RevokeSecurityGroupIngressError,
        run_instances::RunInstancesError, terminate_instances::TerminateInstancesError,
    },
    waiters::instance_running::WaitUntilInstanceRunningError,
};
use aws_sdk_iam::operation::simulate_principal_policy::SimulatePrincipalPolicyError;
use aws_sdk_pricing::operation::get_products::GetProductsError;
use aws_sdk_ssm::operation::get_parameter::GetParameterError;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityError;
use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{CloudError, Error as CoreError, ErrorKind},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AwsError>;

#[derive(Debug, Error)]
pub enum AwsError {
    #[error("{operation} rejected with {}: {}", .code.as_deref().unwrap_or("no error code"), .message.as_deref().unwrap_or(.detail.as_str()))]
    Rejected {
        operation: &'static str,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        detail: String,
    },

    #[error("{operation} could not reach AWS: {detail}")]
    Transport {
        operation: &'static str,
        detail: String,
    },

    #[error("{resource} is missing {field}")]
    MissingField {
        resource: &'static str,
        field: &'static str,
    },

    #[error("no availability zones found in the region")]
    NoAvailabilityZones,

    #[error("virtual private cloud {vpc_id} has no IPv6 CIDR block")]
    MissingVpcIpv6Cidr { vpc_id: String },

    #[error("invalid IPv6 CIDR block {cidr}")]
    InvalidIpv6Cidr { cidr: String },

    #[error("virtual private cloud {vpc_id} has no main route table")]
    MissingMainRouteTable { vpc_id: String },

    #[error("instance did not reach the running state")]
    InstanceNotRunning(#[source] Box<WaitUntilInstanceRunningError>),

    #[error("instance {instance_id} has no public IPv4 address")]
    MissingPublicIpv4 { instance_id: String },

    #[error("instance {instance_id} has no public IPv6 address")]
    MissingPublicIpv6 { instance_id: String },

    #[error("AMI {name} not found")]
    AmiNotFound { name: &'static str },

    #[error("no on-demand USD price found for {component}")]
    PriceUnavailable { component: String },

    #[error("invalid pricing filter")]
    InvalidPricingFilter(#[from] BuildError),

    #[error(transparent)]
    Core(#[from] CoreError),
}

impl AwsError {
    pub fn has_code(&self, expected_code: &str) -> bool {
        matches!(self, Self::Rejected { code: Some(code), .. } if code == expected_code)
    }
}

impl CloudError for AwsError {
    fn provider(&self) -> CloudProviderName {
        CloudProviderName::Aws
    }

    fn kind(&self) -> ErrorKind {
        match self {
            Self::Rejected { status, code, .. } => code
                .as_deref()
                .and_then(classify_aws_code)
                .unwrap_or_else(|| ErrorKind::from_http_status(*status)),
            Self::Transport { .. } | Self::InstanceNotRunning(_) => ErrorKind::Transient,
            Self::AmiNotFound { .. } => ErrorKind::NotFound,
            Self::InvalidPricingFilter(_) | Self::Core(_) => ErrorKind::Unknown,
            Self::MissingField { .. }
            | Self::NoAvailabilityZones
            | Self::MissingVpcIpv6Cidr { .. }
            | Self::InvalidIpv6Cidr { .. }
            | Self::MissingMainRouteTable { .. }
            | Self::MissingPublicIpv4 { .. }
            | Self::MissingPublicIpv6 { .. }
            | Self::PriceUnavailable { .. } => ErrorKind::Cloud,
        }
    }

    fn operation(&self) -> Option<String> {
        match self {
            Self::Rejected { operation, .. } | Self::Transport { operation, .. } => {
                Some((*operation).to_string())
            }
            _ => None,
        }
    }

    fn code(&self) -> Option<String> {
        match self {
            Self::Rejected { code, .. } => code.clone(),
            _ => None,
        }
    }
}

impl From<AwsError> for CoreError {
    fn from(error: AwsError) -> Self {
        match error {
            AwsError::Core(core_error) => core_error,
            other => Self::Cloud(Box::new(other)),
        }
    }
}

impl From<WaitUntilInstanceRunningError> for AwsError {
    fn from(error: WaitUntilInstanceRunningError) -> Self {
        Self::InstanceNotRunning(Box::new(error))
    }
}

macro_rules! convert_operation_errors {
    ($($error_type:ty => $operation:literal),+ $(,)?) => {
        $(
            impl From<SdkError<$error_type>> for AwsError {
                fn from(sdk_error: SdkError<$error_type>) -> Self {
                    convert_sdk_error($operation, sdk_error)
                }
            }
        )+
    };
}

convert_operation_errors! {
    DescribeVpcsError => "ec2:DescribeVpcs",
    CreateVpcError => "ec2:CreateVpc",
    DescribeSubnetsError => "ec2:DescribeSubnets",
    DescribeAvailabilityZonesError => "ec2:DescribeAvailabilityZones",
    CreateSubnetError => "ec2:CreateSubnet",
    ModifySubnetAttributeError => "ec2:ModifySubnetAttribute",
    DescribeSecurityGroupsError => "ec2:DescribeSecurityGroups",
    CreateSecurityGroupError => "ec2:CreateSecurityGroup",
    AuthorizeSecurityGroupIngressError => "ec2:AuthorizeSecurityGroupIngress",
    RevokeSecurityGroupIngressError => "ec2:RevokeSecurityGroupIngress",
    DescribeRouteTablesError => "ec2:DescribeRouteTables",
    DescribeInternetGatewaysError => "ec2:DescribeInternetGateways",
    CreateInternetGatewayError => "ec2:CreateInternetGateway",
    AttachInternetGatewayError => "ec2:AttachInternetGateway",
    CreateRouteError => "ec2:CreateRoute",
    CreateTagsError => "ec2:CreateTags",
    RunInstancesError => "ec2:RunInstances",
    TerminateInstancesError => "ec2:TerminateInstances",
    DescribeInstancesError => "ec2:DescribeInstances",
    DescribeRegionsError => "ec2:DescribeRegions",
    GetParameterError => "ssm:GetParameter",
    GetCallerIdentityError => "sts:GetCallerIdentity",
    SimulatePrincipalPolicyError => "iam:SimulatePrincipalPolicy",
    GetProductsError => "pricing:GetProducts",
}

fn classify_aws_code(code: &str) -> Option<ErrorKind> {
    match code {
        "UnauthorizedOperation" | "AccessDenied" | "AccessDeniedException" => {
            Some(ErrorKind::Authorization)
        }
        "AuthFailure"
        | "InvalidClientTokenId"
        | "SignatureDoesNotMatch"
        | "UnrecognizedClientException"
        | "ExpiredToken" => Some(ErrorKind::Authentication),
        "ThrottlingException" | "RequestLimitExceeded" => Some(ErrorKind::Quota),
        _ if code.ends_with(".NotFound") => Some(ErrorKind::NotFound),
        _ => None,
    }
}

fn convert_sdk_error<E>(operation: &'static str, sdk_error: SdkError<E>) -> AwsError
where
    E: ProvideErrorMetadata + StdError + Send + Sync + 'static,
{
    let detail = DisplayErrorContext(&sdk_error).to_string();
    match sdk_error {
        SdkError::ServiceError(service_error) => {
            let status = service_error.raw().status().as_u16();
            let error = service_error.into_err();
            AwsError::Rejected {
                operation,
                status,
                code: error.code().map(str::to_string),
                message: error.message().map(str::to_string),
                detail,
            }
        }
        SdkError::ResponseError(response_error) => AwsError::Rejected {
            operation,
            status: response_error.raw().status().as_u16(),
            code: None,
            message: None,
            detail,
        },
        _ => AwsError::Transport { operation, detail },
    }
}
