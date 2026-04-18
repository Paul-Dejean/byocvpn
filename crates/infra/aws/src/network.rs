use std::{net::Ipv6Addr, str::FromStr};

use aws_sdk_ec2::{
    Client as Ec2Client,
    error::ProvideErrorMetadata,
    types::{
        AttributeBooleanValue, Filter, IpPermission, IpRange, Ipv6Range, ResourceType, Tag,
        TagSpecification,
    },
};
use byocvpn_core::error::{NetworkProvisioningError, Result};
use log::*;

use crate::{
    aws_error::sdk_error_message,
    constants::{IPV4_ALL_CIDR, IPV6_ALL_CIDR},
};

async fn create_security_group(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    group_name: &str,
    description: &str,
) -> Result<String> {
    let create_response = ec2_client
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

    let group_id = create_response
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

async fn get_security_group_by_name(
    ec2_client: &Ec2Client,
    name: &str,
) -> Result<Option<String>> {
    let filters = Filter::builder().name("group-name").values(name).build();

    let response = ec2_client
        .describe_security_groups()
        .filters(filters)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let group_id = response
        .security_groups()
        .first()
        .and_then(|security_group| security_group.group_id())
        .map(|group_id| group_id.to_string());

    Ok(group_id)
}

fn build_desired_ingress_rules() -> Vec<IpPermission> {
    vec![
        IpPermission::builder()
            .ip_protocol("udp")
            .from_port(51820)
            .to_port(51820)
            .ip_ranges(IpRange::builder().cidr_ip(IPV4_ALL_CIDR).build())
            .build(),
        IpPermission::builder()
            .ip_protocol("tcp")
            .from_port(51820)
            .to_port(51820)
            .ip_ranges(IpRange::builder().cidr_ip(IPV4_ALL_CIDR).build())
            .build(),
        IpPermission::builder()
            .ip_protocol("udp")
            .from_port(51820)
            .to_port(51820)
            .ipv6_ranges(Ipv6Range::builder().cidr_ipv6(IPV6_ALL_CIDR).build())
            .build(),
        IpPermission::builder()
            .ip_protocol("tcp")
            .from_port(51820)
            .to_port(51820)
            .ipv6_ranges(Ipv6Range::builder().cidr_ipv6(IPV6_ALL_CIDR).build())
            .build(),
    ]
}

fn ingress_rule_matches_desired(
    existing: &IpPermission,
    desired: &IpPermission,
) -> bool {
    existing.ip_protocol() == desired.ip_protocol()
        && existing.from_port() == desired.from_port()
        && existing.to_port() == desired.to_port()
        && existing
            .ip_ranges()
            .iter()
            .any(|existing_range| {
                desired
                    .ip_ranges()
                    .iter()
                    .any(|desired_range| existing_range.cidr_ip() == desired_range.cidr_ip())
            })
        && existing
            .ipv6_ranges()
            .iter()
            .any(|existing_range| {
                desired
                    .ipv6_ranges()
                    .iter()
                    .any(|desired_range| existing_range.cidr_ipv6() == desired_range.cidr_ipv6())
            })
}

fn existing_rule_is_desired(
    existing: &IpPermission,
    desired_rules: &[IpPermission],
) -> bool {
    desired_rules
        .iter()
        .any(|desired| ingress_rule_matches_desired(existing, desired))
}

async fn patch_security_group_rules(
    ec2_client: &Ec2Client,
    security_group_id: &str,
) -> Result<()> {
    let describe_response = ec2_client
        .describe_security_groups()
        .group_ids(security_group_id)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let current_rules: Vec<IpPermission> = describe_response
        .security_groups()
        .first()
        .map(|security_group| security_group.ip_permissions().to_vec())
        .unwrap_or_default();

    let desired_rules = build_desired_ingress_rules();

    let rules_to_add: Vec<IpPermission> = desired_rules
        .iter()
        .filter(|desired| {
            !current_rules
                .iter()
                .any(|existing| ingress_rule_matches_desired(existing, desired))
        })
        .cloned()
        .collect();

    let rules_to_remove: Vec<IpPermission> = current_rules
        .into_iter()
        .filter(|existing| !existing_rule_is_desired(existing, &desired_rules))
        .collect();

    if !rules_to_add.is_empty() {
        let mut authorize_builder = ec2_client
            .authorize_security_group_ingress()
            .group_id(security_group_id);
        for rule in rules_to_add {
            authorize_builder = authorize_builder.ip_permissions(rule);
        }
        authorize_builder
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::SecurityGroupRuleConfigurationFailed {
                reason: sdk_error_message(&error),
            })?;
    }

    if !rules_to_remove.is_empty() {
        let mut revoke_builder = ec2_client
            .revoke_security_group_ingress()
            .group_id(security_group_id);
        for rule in rules_to_remove {
            revoke_builder = revoke_builder.ip_permissions(rule);
        }
        revoke_builder
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::SecurityGroupRuleConfigurationFailed {
                reason: sdk_error_message(&error),
            })?;
    }

    Ok(())
}

pub async fn ensure_security_group(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    name: &str,
    description: &str,
) -> Result<String> {
    if let Some(security_group_id) = get_security_group_by_name(ec2_client, name).await? {
        patch_security_group_rules(ec2_client, &security_group_id).await?;
        return Ok(security_group_id);
    }
    create_security_group(ec2_client, vpc_id, name, description).await
}

async fn create_vpc(
    ec2_client: &Ec2Client,
    cidr_block: &str,
    name: &str,
) -> Result<String> {
    let tag_spec = TagSpecification::builder()
        .resource_type(ResourceType::Vpc)
        .tags(Tag::builder().key("Name").value(name).build())
        .build();

    let response = ec2_client
        .create_vpc()
        .cidr_block(cidr_block)
        .amazon_provided_ipv6_cidr_block(true)
        .tag_specifications(tag_spec)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
            reason: sdk_error_message(&error),
        })?;

    let vpc_id = response
        .vpc()
        .and_then(|vpc| vpc.vpc_id())
        .ok_or_else(|| NetworkProvisioningError::MissingVpcIdentifier)?;

    info!("Created VPC: {}", vpc_id);
    Ok(vpc_id.to_string())
}

async fn get_vpc_by_name(ec2_client: &Ec2Client, name: &str) -> Result<Option<String>> {
    let filter = Filter::builder().name("tag:Name").values(name).build();

    let response = ec2_client
        .describe_vpcs()
        .filters(filter)
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let vpc_id = response
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(|vpc_id| vpc_id.to_string());

    Ok(vpc_id)
}

pub async fn ensure_vpc(ec2: &Ec2Client, cidr: &str, name: &str) -> Result<String> {
    if let Some(vpc_id) = get_vpc_by_name(ec2, name).await? {
        return Ok(vpc_id);
    }
    create_vpc(ec2, cidr, name).await
}

async fn create_subnet(
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

    let response = ec2_client
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

    let subnet_id = response
        .subnet()
        .and_then(|subnet| subnet.subnet_id())
        .ok_or_else(|| NetworkProvisioningError::MissingSubnetIdentifier)?;

    info!("Created Subnet: {}", subnet_id);
    Ok(subnet_id.to_string())
}

async fn get_subnets_in_vpc(ec2_client: &Ec2Client, vpc_id: &str) -> Result<Vec<String>> {
    let response = ec2_client
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

    let subnet_ids = response
        .subnets()
        .iter()
        .filter_map(|subnet| subnet.subnet_id())
        .map(|subnet_id| subnet_id.to_string())
        .collect();

    Ok(subnet_ids)
}

pub async fn ensure_subnet(
    ec2: &Ec2Client,
    vpc_id: &str,
    cidr: &str,
    subnet_name: &str,
) -> Result<String> {
    let existing = get_subnets_in_vpc(ec2, vpc_id).await?;
    if let Some(subnet_id) = existing.into_iter().next() {
        return Ok(subnet_id);
    }
    let availability_zones = list_availability_zones(ec2).await?;
    let availability_zone = availability_zones.first().ok_or(
        NetworkProvisioningError::NetworkQueryFailed {
            reason: "no availability zones found in region".to_string(),
        },
    )?;
    let vpc_ipv6_cidr = get_vpc_ipv6_block(ec2, vpc_id).await?;
    let ipv6_cidr = carve_ipv6_subnet(&vpc_ipv6_cidr, 0)?;
    let subnet_id = create_subnet(ec2, vpc_id, cidr, &ipv6_cidr, availability_zone, subnet_name).await?;
    enable_auto_ip_assign(ec2, &subnet_id).await?;
    Ok(subnet_id)
}

pub(super) async fn list_availability_zones(ec2_client: &Ec2Client) -> Result<Vec<String>> {
    let response = ec2_client
        .describe_availability_zones()
        .send()
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: sdk_error_message(&error),
        })?;

    let availability_zones = response
        .availability_zones()
        .iter()
        .filter_map(|availability_zone| availability_zone.zone_name())
        .map(|availability_zone| availability_zone.to_string())
        .collect();

    Ok(availability_zones)
}

pub(super) async fn get_vpc_ipv6_block(ec2_client: &Ec2Client, vpc_id: &str) -> Result<String> {
    let response = ec2_client
        .describe_vpcs()
        .vpc_ids(vpc_id)
        .send()
        .await
        .map_err(|_error| NetworkProvisioningError::MissingVpcIdentifier)?;

    let cidr = response
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

async fn get_internet_gateway(ec2: &Ec2Client, vpc_id: &str) -> Result<Option<String>> {
    let response = ec2
        .describe_internet_gateways()
        .filters(
            Filter::builder()
                .name("attachment.vpc-id")
                .values(vpc_id)
                .build(),
        )
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::InternetGatewayOperationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    let igw_id = response
        .internet_gateways()
        .first()
        .and_then(|igw| igw.internet_gateway_id())
        .map(|igw_id| igw_id.to_string());

    Ok(igw_id)
}

async fn create_internet_gateway(ec2: &Ec2Client, vpc_id: &str) -> Result<String> {
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

    info!("Internet Gateway {igw_id} attached to VPC {vpc_id}");
    Ok(igw_id.to_string())
}

pub async fn ensure_internet_gateway(
    ec2: &Ec2Client,
    vpc_id: &str,
    igw_name: &str,
    route_table_name: &str,
) -> Result<String> {
    let igw_id = if let Some(existing_id) = get_internet_gateway(ec2, vpc_id).await? {
        existing_id
    } else {
        create_internet_gateway(ec2, vpc_id).await?
    };
    let route_table_id = find_main_route_table(ec2, vpc_id).await?;
    tag_resource_with_name(ec2, &route_table_id, route_table_name).await?;
    tag_resource_with_name(ec2, &igw_id, igw_name).await?;
    add_igw_routes_to_table(ec2, &route_table_id, &igw_id).await?;
    Ok(igw_id)
}

fn is_route_already_exists_error<E: ProvideErrorMetadata>(error: &aws_sdk_ec2::error::SdkError<E>) -> bool {
    if let aws_sdk_ec2::error::SdkError::ServiceError(service_error) = error {
        return service_error.err().code() == Some("RouteAlreadyExists");
    }
    false
}

pub(super) async fn add_igw_routes_to_table(
    ec2: &Ec2Client,
    route_table_id: &str,
    igw_id: &str,
) -> Result<()> {
    let ipv4_result = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_cidr_block(IPV4_ALL_CIDR)
        .gateway_id(igw_id)
        .send()
        .await;

    if let Err(error) = ipv4_result {
        if !is_route_already_exists_error(&error) {
            return Err(NetworkProvisioningError::RouteTableOperationFailed {
                reason: sdk_error_message(&error),
            }
            .into());
        }
    }

    let ipv6_result = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_ipv6_cidr_block(IPV6_ALL_CIDR)
        .gateway_id(igw_id)
        .send()
        .await;

    if let Err(error) = ipv6_result {
        if !is_route_already_exists_error(&error) {
            return Err(NetworkProvisioningError::RouteTableOperationFailed {
                reason: sdk_error_message(&error),
            }
            .into());
        }
    }

    info!("Added default routes to route table: {}", route_table_id);
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

    let response = ec2
        .describe_route_tables()
        .filters(filters)
        .send()
        .await
        .map_err(
            |error| NetworkProvisioningError::RouteTableOperationFailed {
                reason: sdk_error_message(&error),
            },
        )?;

    let route_table_id = response
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

    Ok(route_table_id.to_string())
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
