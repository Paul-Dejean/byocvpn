use aws_sdk_ec2::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use byocvpn_core::{cloud_provider::PermissionStatus, error::Result};
use log::*;

use crate::ami;
use crate::constants::{
    IPV4_ALL_CIDR, SECURITY_GROUP_NAME, SUBNET_CIDR_BLOCK, VPC_CIDR_BLOCK, VPC_NAME,
};

const PLACEHOLDER_INSTANCE_ID: &str = "i-0ba8dd7fe03dfbb57";
const DRY_RUN_AUTHORIZED_CODES: &[&str] = &["DryRunOperation"];
const TERMINATE_PLACEHOLDER_AUTHORIZED_CODES: &[&str] =
    &["DryRunOperation", "InvalidInstanceID.NotFound"];

fn is_dry_run_authorized<T, E>(operation: &str, result: std::result::Result<T, SdkError<E>>) -> bool
where
    E: ProvideErrorMetadata,
{
    is_dry_run_authorized_with_codes(operation, result, DRY_RUN_AUTHORIZED_CODES)
}

fn is_dry_run_authorized_with_codes<T, E>(
    operation: &str,
    result: std::result::Result<T, SdkError<E>>,
    authorized_codes: &[&str],
) -> bool
where
    E: ProvideErrorMetadata,
{
    match result {
        Ok(_) => {
            info!("permission check {operation}: authorized (request succeeded)");
            true
        }
        Err(SdkError::ServiceError(service_error)) => {
            let error_code = service_error.err().code().unwrap_or("unknown");
            let error_message = service_error.err().message().unwrap_or("");
            let authorized = is_authorized_error_code(service_error.err().code(), authorized_codes);
            if authorized {
                info!(
                    "permission check {operation}: authorized (code={error_code}, message={error_message})"
                );
            } else {
                warn!(
                    "permission check {operation}: denied (code={error_code}, message={error_message})"
                );
            }
            authorized
        }
        Err(other_error) => {
            warn!(
                "permission check {operation}: inconclusive (code={:?})",
                other_error.code()
            );
            false
        }
    }
}

fn is_authorized_error_code(code: Option<&str>, authorized_codes: &[&str]) -> bool {
    code.is_some_and(|error_code| authorized_codes.contains(&error_code))
}

async fn get_any_instance_id(ec2_client: &Ec2Client) -> Option<String> {
    let response = ec2_client
        .describe_instances()
        .max_results(5)
        .send()
        .await
        .ok()?;
    response
        .reservations()
        .iter()
        .flat_map(|reservation| reservation.instances())
        .find_map(|instance| instance.instance_id())
        .map(|instance_id| instance_id.to_string())
}

async fn is_terminate_instances_authorized(ec2_client: &Ec2Client) -> bool {
    let (instance_id, authorized_codes) = match get_any_instance_id(ec2_client).await {
        Some(existing_instance_id) => (existing_instance_id, DRY_RUN_AUTHORIZED_CODES),
        None => (
            PLACEHOLDER_INSTANCE_ID.to_string(),
            TERMINATE_PLACEHOLDER_AUTHORIZED_CODES,
        ),
    };
    is_dry_run_authorized_with_codes(
        "ec2:TerminateInstances",
        ec2_client
            .terminate_instances()
            .instance_ids(instance_id)
            .dry_run(true)
            .send()
            .await,
        authorized_codes,
    )
}

async fn get_default_vpc_id(ec2_client: &Ec2Client) -> Option<String> {
    let response = ec2_client
        .describe_vpcs()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("isDefault")
                .values("true")
                .build(),
        )
        .send()
        .await
        .ok()?;
    response
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(|vpc_id| vpc_id.to_string())
}

async fn get_default_security_group_id(ec2_client: &Ec2Client) -> Option<String> {
    let response = ec2_client
        .describe_security_groups()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("group-name")
                .values("default")
                .build(),
        )
        .send()
        .await
        .ok()?;
    response
        .security_groups()
        .first()
        .and_then(|security_group| security_group.group_id())
        .map(|group_id| group_id.to_string())
}

pub(super) async fn verify_permissions(
    ec2_client: &Ec2Client,
    ssm_client: &SsmClient,
) -> Result<Vec<PermissionStatus>> {
    let ec2_run_instances = match ami::get_al2023_ami(ssm_client).await {
        Ok(image_id) => is_dry_run_authorized(
            "ec2:RunInstances",
            ec2_client
                .run_instances()
                .image_id(image_id)
                .max_count(1)
                .min_count(1)
                .dry_run(true)
                .send()
                .await,
        ),
        Err(error) => {
            warn!("permission check ec2:RunInstances: could not resolve AMI ({error}); failing");
            false
        }
    };

    let ec2_terminate_instances = is_terminate_instances_authorized(ec2_client).await;

    let ec2_create_vpc = is_dry_run_authorized(
        "ec2:CreateVpc",
        ec2_client
            .create_vpc()
            .cidr_block(VPC_CIDR_BLOCK)
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_create_subnet = is_dry_run_authorized(
        "ec2:CreateSubnet",
        ec2_client
            .create_subnet()
            .vpc_id("vpc-12345678")
            .cidr_block(SUBNET_CIDR_BLOCK)
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_create_security_group = is_dry_run_authorized(
        "ec2:CreateSecurityGroup",
        ec2_client
            .create_security_group()
            .group_name(SECURITY_GROUP_NAME)
            .description("permission check")
            .vpc_id("vpc-12345678")
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_create_tags = match get_default_vpc_id(ec2_client).await {
        Some(vpc_id) => is_dry_run_authorized(
            "ec2:CreateTags",
            ec2_client
                .create_tags()
                .resources(vpc_id)
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("Name")
                        .value(VPC_NAME)
                        .build(),
                )
                .dry_run(true)
                .send()
                .await,
        ),
        None => {
            warn!("permission check ec2:CreateTags: no default VPC found; treating as authorized");
            true
        }
    };

    let security_group_ingress_permission = aws_sdk_ec2::types::IpPermission::builder()
        .ip_protocol("tcp")
        .from_port(22)
        .to_port(22)
        .ip_ranges(
            aws_sdk_ec2::types::IpRange::builder()
                .cidr_ip(IPV4_ALL_CIDR)
                .build(),
        )
        .build();

    let default_security_group_id = get_default_security_group_id(ec2_client).await;

    let ec2_authorize_security_group_ingress = match &default_security_group_id {
        Some(group_id) => is_dry_run_authorized(
            "ec2:AuthorizeSecurityGroupIngress",
            ec2_client
                .authorize_security_group_ingress()
                .group_id(group_id)
                .ip_permissions(security_group_ingress_permission.clone())
                .dry_run(true)
                .send()
                .await,
        ),
        None => {
            warn!(
                "permission check ec2:AuthorizeSecurityGroupIngress: no default security group found; treating as authorized"
            );
            true
        }
    };

    let ec2_revoke_security_group_ingress = match &default_security_group_id {
        Some(group_id) => is_dry_run_authorized(
            "ec2:RevokeSecurityGroupIngress",
            ec2_client
                .revoke_security_group_ingress()
                .group_id(group_id)
                .ip_permissions(security_group_ingress_permission)
                .dry_run(true)
                .send()
                .await,
        ),
        None => {
            warn!(
                "permission check ec2:RevokeSecurityGroupIngress: no default security group found; treating as authorized"
            );
            true
        }
    };

    let ec2_describe_instances = is_dry_run_authorized(
        "ec2:DescribeInstances",
        ec2_client.describe_instances().dry_run(true).send().await,
    );

    let ec2_describe_vpcs = is_dry_run_authorized(
        "ec2:DescribeVpcs",
        ec2_client.describe_vpcs().dry_run(true).send().await,
    );

    let ec2_describe_subnets = is_dry_run_authorized(
        "ec2:DescribeSubnets",
        ec2_client.describe_subnets().dry_run(true).send().await,
    );

    let ec2_describe_security_groups = is_dry_run_authorized(
        "ec2:DescribeSecurityGroups",
        ec2_client
            .describe_security_groups()
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_describe_availability_zones = is_dry_run_authorized(
        "ec2:DescribeAvailabilityZones",
        ec2_client
            .describe_availability_zones()
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_describe_regions = is_dry_run_authorized(
        "ec2:DescribeRegions",
        ec2_client.describe_regions().dry_run(true).send().await,
    );

    let ec2_create_internet_gateway = is_dry_run_authorized(
        "ec2:CreateInternetGateway",
        ec2_client
            .create_internet_gateway()
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_describe_internet_gateways = is_dry_run_authorized(
        "ec2:DescribeInternetGateways",
        ec2_client
            .describe_internet_gateways()
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_attach_internet_gateway = is_dry_run_authorized(
        "ec2:AttachInternetGateway",
        ec2_client
            .attach_internet_gateway()
            .internet_gateway_id("igw-12345678")
            .vpc_id("vpc-12345678")
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_describe_route_tables = is_dry_run_authorized(
        "ec2:DescribeRouteTables",
        ec2_client
            .describe_route_tables()
            .dry_run(true)
            .send()
            .await,
    );

    let ec2_create_route = is_dry_run_authorized(
        "ec2:CreateRoute",
        ec2_client
            .create_route()
            .route_table_id("rtb-12345678")
            .destination_cidr_block(IPV4_ALL_CIDR)
            .gateway_id("igw-12345678")
            .dry_run(true)
            .send()
            .await,
    );

    let ssm_get_parameter_result = ssm_client
        .get_parameter()
        .name(ami::AL2023_AMI_SSM_PARAMETER)
        .send()
        .await;
    let ssm_get_parameter = match &ssm_get_parameter_result {
        Ok(_) => {
            info!("permission check ssm:GetParameter: authorized (request succeeded)");
            true
        }
        Err(error) => {
            warn!(
                "permission check ssm:GetParameter: denied (code={:?}, message={:?})",
                error.code(),
                error.message()
            );
            false
        }
    };

    let permissions = [
        ("ec2:RunInstances", ec2_run_instances),
        ("ec2:TerminateInstances", ec2_terminate_instances),
        ("ec2:CreateVpc", ec2_create_vpc),
        ("ec2:CreateSubnet", ec2_create_subnet),
        ("ec2:CreateSecurityGroup", ec2_create_security_group),
        ("ec2:CreateTags", ec2_create_tags),
        (
            "ec2:AuthorizeSecurityGroupIngress",
            ec2_authorize_security_group_ingress,
        ),
        (
            "ec2:RevokeSecurityGroupIngress",
            ec2_revoke_security_group_ingress,
        ),
        ("ec2:DescribeInstances", ec2_describe_instances),
        ("ec2:DescribeVpcs", ec2_describe_vpcs),
        ("ec2:DescribeSubnets", ec2_describe_subnets),
        ("ec2:DescribeSecurityGroups", ec2_describe_security_groups),
        (
            "ec2:DescribeAvailabilityZones",
            ec2_describe_availability_zones,
        ),
        (
            "ec2:DescribeInternetGateways",
            ec2_describe_internet_gateways,
        ),
        ("ec2:CreateInternetGateway", ec2_create_internet_gateway),
        ("ec2:AttachInternetGateway", ec2_attach_internet_gateway),
        ("ec2:DescribeRouteTables", ec2_describe_route_tables),
        ("ec2:CreateRoute", ec2_create_route),
        ("ec2:DescribeRegions", ec2_describe_regions),
        ("ssm:GetParameter", ssm_get_parameter),
    ]
    .into_iter()
    .map(|(permission, granted)| PermissionStatus {
        permission: permission.to_string(),
        granted,
    })
    .collect::<Vec<_>>();

    Ok(permissions)
}
