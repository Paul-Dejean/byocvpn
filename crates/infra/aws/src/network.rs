use std::{net::Ipv6Addr, str::FromStr};

use aws_sdk_ec2::{
    Client as Ec2Client,
    types::{
        AttributeBooleanValue, Filter, IpPermission, IpRange, Ipv6Range, ResourceType, Subnet, Tag,
        TagSpecification,
    },
};
use byocvpn_core::error::{NetworkProvisioningError, Result};
use log::*;

use crate::aws_error::sdk_error_message;
use crate::constants::{IPV4_ALL_CIDR, IPV6_ALL_CIDR};

pub(super) async fn create_security_group(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    group_name: &str,
    description: &str,
) -> Result<String> {
    let create_resp = ec2_client
        .create_security_group()
        .vpc_id(vpc_id)
        .group_name(group_name)
        .description(description)
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    let group_id = create_resp
        .group_id()
        .ok_or(NetworkProvisioningError::MissingSecurityGroupIdentifier)?
        .to_string();

    info!("Created security group with ID: {}", group_id);

    ec2_client
        .authorize_security_group_ingress()
        .group_id(&group_id)
        .ip_permissions(
            IpPermission::builder()
                .ip_protocol("udp")
                .from_port(51820)
                .to_port(51820)
                .ip_ranges(IpRange::builder().cidr_ip(IPV4_ALL_CIDR).build())
                .ipv6_ranges(Ipv6Range::builder().cidr_ipv6(IPV6_ALL_CIDR).build())
                .build(),
        )
        .ip_permissions(
            IpPermission::builder()
                .ip_protocol("tcp")
                .from_port(51820)
                .to_port(51820)
                .ip_ranges(IpRange::builder().cidr_ip(IPV4_ALL_CIDR).build())
                .ipv6_ranges(Ipv6Range::builder().cidr_ipv6(IPV6_ALL_CIDR).build())
                .build(),
        )
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupRuleConfigurationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    Ok(group_id)
}

pub(super) async fn get_security_group_by_name(
    ec2_client: &Ec2Client,
    group_name: &str,
) -> Result<Option<String>> {
    let filters = Filter::builder()
        .name("group-name")
        .values(group_name)
        .build();

    let resp = ec2_client
        .describe_security_groups()
        .filters(filters)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let group_id = resp
        .security_groups()
        .first()
        .and_then(|security_group| security_group.group_id())
        .map(|group_id| group_id.to_string());

    Ok(group_id)
}

pub(super) async fn create_vpc(
    ec2_client: &Ec2Client,
    cidr_block: &str,
    name: &str,
) -> Result<String> {
    let tag_spec = TagSpecification::builder()
        .resource_type(ResourceType::Vpc)
        .tags(Tag::builder().key("Name").value(name).build())
        .build();

    let resp = ec2_client
        .create_vpc()
        .cidr_block(cidr_block)
        .amazon_provided_ipv6_cidr_block(true)
        .tag_specifications(tag_spec)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
            reason: sdk_error_message(&error),
        })?;

    let vpc_id = resp
        .vpc()
        .and_then(|vpc| vpc.vpc_id())
        .ok_or_else(|| NetworkProvisioningError::MissingVpcIdentifier)?;

    info!("Created VPC: {}", vpc_id);
    Ok(vpc_id.to_string())
}

pub(super) async fn get_vpc_by_name(ec2_client: &Ec2Client, name: &str) -> Result<Option<String>> {
    let filter = Filter::builder().name("tag:Name").values(name).build();

    let resp = ec2_client
        .describe_vpcs()
        .filters(filter)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let vpc_id = resp
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(|vpc_id| vpc_id.to_string());

    Ok(vpc_id)
}

pub(super) async fn create_subnet(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    cidr_block: &str,
    ipv6_cidr_block: &str,
    availability_zone: &str,
    name: &str,
) -> Result<String> {
    let tag_spec = TagSpecification::builder()
        .resource_type(ResourceType::Subnet)
        .tags(Tag::builder().key("Name").value(name).build())
        .build();

    let resp = ec2_client
        .create_subnet()
        .vpc_id(vpc_id)
        .cidr_block(cidr_block)
        .ipv6_cidr_block(ipv6_cidr_block)
        .availability_zone(availability_zone)
        .tag_specifications(tag_spec)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
            reason: sdk_error_message(&error),
        })?;

    let subnet_id = resp
        .subnet()
        .and_then(|subnet| subnet.subnet_id())
        .ok_or_else(|| NetworkProvisioningError::MissingSubnetIdentifier)?;

    info!("Created Subnet: {}", subnet_id);
    Ok(subnet_id.to_string())
}

pub(super) async fn list_availability_zones(ec2_client: &Ec2Client) -> Result<Vec<String>> {
    let resp = ec2_client
        .describe_availability_zones()
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let availability_zones = resp
        .availability_zones()
        .iter()
        .filter_map(|availability_zone| availability_zone.zone_name())
        .map(|availability_zone| availability_zone.to_string())
        .collect();

    Ok(availability_zones)
}

pub(super) async fn get_vpc_ipv6_block(ec2_client: &Ec2Client, vpc_id: &str) -> Result<String> {
    let resp = ec2_client
        .describe_vpcs()
        .vpc_ids(vpc_id)
        .send()
        .await
        .map_err(|_error| NetworkProvisioningError::MissingVpcIdentifier)?;

    let cidr = resp
        .vpcs()
        .iter()
        .flat_map(|vpc| vpc.ipv6_cidr_block_association_set())
        .filter_map(|assoc| assoc.ipv6_cidr_block())
        .next()
        .ok_or_else(|| NetworkProvisioningError::MissingVpcIpv6Cidr {
            vpc_id: vpc_id.to_string(),
        })?;

    Ok(cidr.to_string())
}

pub(super) fn carve_ipv6_subnet(base_cidr: &str, index: u8) -> Result<String> {
    let (base_ip, _prefix) =
        base_cidr
            .split_once('/')
            .ok_or_else(|| NetworkProvisioningError::InvalidIpv6Cidr {
                cidr: base_cidr.to_string(),
            })?;
    let mut bytes = Ipv6Addr::from_str(base_ip)
        .map_err(|_| NetworkProvisioningError::InvalidIpv6Cidr {
            cidr: base_ip.to_string(),
        })?
        .octets();

    bytes[7] = index;

    let subnet = Ipv6Addr::from(bytes);
    Ok(format!("{}/64", subnet))
}

pub(super) async fn create_and_attach_igw(ec2: &Ec2Client, vpc_id: &str) -> Result<String> {
    let igw = ec2
        .create_internet_gateway()
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::InternetGatewayOperationFailed {
                reason: sdk_error_message(&error),
            },
        )?;
    let igw_id = igw
        .internet_gateway()
        .and_then(|gateway| gateway.internet_gateway_id())
        .ok_or_else(|| NetworkProvisioningError::MissingInternetGatewayIdentifier)?;

    ec2.attach_internet_gateway()
        .internet_gateway_id(igw_id)
        .vpc_id(vpc_id)
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::InternetGatewayOperationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    info!("🌐 Internet Gateway {igw_id} attached to VPC {vpc_id}");
    Ok(igw_id.to_string())
}

pub(super) async fn add_igw_routes_to_table(
    ec2: &Ec2Client,
    route_table_id: &str,
    igw_id: &str,
) -> Result<()> {
    let _ = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_cidr_block(IPV4_ALL_CIDR)
        .gateway_id(igw_id)
        .send()
        .await;

    let _ = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_ipv6_cidr_block(IPV6_ALL_CIDR)
        .gateway_id(igw_id)
        .send()
        .await;

    info!("✅ Added default routes to route table: {}", route_table_id);
    Ok(())
}

pub(super) async fn enable_auto_ip_assign(ec2: &Ec2Client, subnet_id: &str) -> Result<()> {
    ec2.modify_subnet_attribute()
        .subnet_id(subnet_id)
        .map_public_ip_on_launch(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SubnetConfigurationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    ec2.modify_subnet_attribute()
        .subnet_id(subnet_id)
        .assign_ipv6_address_on_creation(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::SubnetConfigurationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    Ok(())
}

pub(super) async fn find_main_route_table(ec2: &Ec2Client, vpc_id: &str) -> Result<String> {
    let filters = Filter::builder().name("vpc-id").values(vpc_id).build();

    let resp = ec2
        .describe_route_tables()
        .filters(filters)
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::RouteTableOperationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    let rt_id = resp
        .route_tables()
        .iter()
        .find(|route_table| {
            route_table
                .associations()
                .iter()
                .any(|assoc| assoc.main().unwrap_or(false))
        })
        .and_then(|route_table| route_table.route_table_id())
        .ok_or_else(|| NetworkProvisioningError::MissingMainRouteTable {
            vpc_id: vpc_id.to_string(),
        })?;

    Ok(rt_id.to_string())
}

pub async fn get_subnets_in_vpc(ec2_client: &Ec2Client, vpc_id: &str) -> Result<Vec<Subnet>> {
    let resp = ec2_client
        .describe_subnets()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("vpc-id")
                .values(vpc_id)
                .build(),
        )
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    Ok(resp.subnets().to_vec())
}

pub async fn tag_resource_with_name(
    ec2_client: &Ec2Client,
    resource_id: &str,
    name: &str,
) -> Result<()> {
    let tag = Tag::builder().key("Name").value(name).build();

    ec2_client
        .create_tags()
        .resources(resource_id)
        .tags(tag)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    Ok(())
}
