use std::{net::Ipv6Addr, str::FromStr};

use aws_sdk_ec2::types::{AttributeBooleanValue, Filter, ResourceType, Tag, TagSpecification};
use log::info;

use crate::{
    client::AwsClient,
    error::{AwsError, Result},
    network::vpc::get_vpc_ipv6_block,
};

const SUBNET_NAME: &str = "byocvpn-subnet";
const SUBNET_CIDR_BLOCK: &str = "10.0.0.0/24";
const SUBNET_IPV6_BLOCK_INDEX: u8 = 0;

pub async fn ensure_subnet(client: &AwsClient, vpc_id: &str) -> Result<String> {
    if let Some(subnet_id) = get_first_subnet_in_vpc(client, vpc_id).await? {
        return Ok(subnet_id);
    }
    let availability_zone = get_first_availability_zone(client).await?;
    let vpc_ipv6_cidr = get_vpc_ipv6_block(client, vpc_id).await?;
    let ipv6_cidr_block = carve_ipv6_subnet(&vpc_ipv6_cidr, SUBNET_IPV6_BLOCK_INDEX)?;
    let subnet_id = create_subnet(client, vpc_id, &ipv6_cidr_block, &availability_zone).await?;
    enable_public_ip_assignment(client, &subnet_id).await?;
    Ok(subnet_id)
}

async fn get_first_subnet_in_vpc(client: &AwsClient, vpc_id: &str) -> Result<Option<String>> {
    let response = client
        .ec2
        .describe_subnets()
        .filters(Filter::builder().name("vpc-id").values(vpc_id).build())
        .send()
        .await?;

    Ok(response
        .subnets()
        .iter()
        .find_map(|subnet| subnet.subnet_id())
        .map(ToString::to_string))
}

async fn get_first_availability_zone(client: &AwsClient) -> Result<String> {
    let response = client.ec2.describe_availability_zones().send().await?;

    response
        .availability_zones()
        .iter()
        .find_map(|availability_zone| availability_zone.zone_name())
        .map(ToString::to_string)
        .ok_or(AwsError::NoAvailabilityZones)
}

fn carve_ipv6_subnet(base_cidr: &str, index: u8) -> Result<String> {
    let (base_ip, _prefix_length) =
        base_cidr
            .split_once('/')
            .ok_or_else(|| AwsError::InvalidIpv6Cidr {
                cidr: base_cidr.to_string(),
            })?;
    let mut octets = Ipv6Addr::from_str(base_ip)
        .map_err(|_| AwsError::InvalidIpv6Cidr {
            cidr: base_ip.to_string(),
        })?
        .octets();
    octets[7] = index;
    Ok(format!("{}/64", Ipv6Addr::from(octets)))
}

async fn create_subnet(
    client: &AwsClient,
    vpc_id: &str,
    ipv6_cidr_block: &str,
    availability_zone: &str,
) -> Result<String> {
    let tag_specification = TagSpecification::builder()
        .resource_type(ResourceType::Subnet)
        .tags(Tag::builder().key("Name").value(SUBNET_NAME).build())
        .build();

    let response = client
        .ec2
        .create_subnet()
        .vpc_id(vpc_id)
        .cidr_block(SUBNET_CIDR_BLOCK)
        .ipv6_cidr_block(ipv6_cidr_block)
        .availability_zone(availability_zone)
        .tag_specifications(tag_specification)
        .send()
        .await?;

    let subnet_id = response
        .subnet()
        .and_then(|subnet| subnet.subnet_id())
        .ok_or(AwsError::MissingResponseField {
            operation: "ec2:CreateSubnet",
            field: "subnetId",
        })?;
    info!("[AWS] Created subnet {subnet_id}");
    Ok(subnet_id.to_string())
}

async fn enable_public_ip_assignment(client: &AwsClient, subnet_id: &str) -> Result<()> {
    client
        .ec2
        .modify_subnet_attribute()
        .subnet_id(subnet_id)
        .map_public_ip_on_launch(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await?;

    client
        .ec2
        .modify_subnet_attribute()
        .subnet_id(subnet_id)
        .assign_ipv6_address_on_creation(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await?;

    Ok(())
}
