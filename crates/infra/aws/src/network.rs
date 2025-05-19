use aws_sdk_ec2::types::AttributeBooleanValue;
use aws_sdk_ec2::types::Subnet;
use aws_sdk_ec2::{
    Client as Ec2Client,
    types::{Filter, IpPermission, IpRange, Ipv6Range, ResourceType, Tag, TagSpecification},
};

use std::net::Ipv6Addr;
use std::str::FromStr;

pub(super) async fn create_security_group(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    group_name: &str,
    description: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let create_resp = ec2_client
        .create_security_group()
        .vpc_id(vpc_id) // Replace with your VPC ID
        .group_name(group_name)
        .description(description)
        .send()
        .await?;

    let group_id = create_resp
        .group_id()
        .ok_or("No security group ID returned")?
        .to_string();

    println!("Created security group with ID: {}", group_id);

    // 2. Authorize SSH ingress from anywhere (0.0.0.0/0)
    ec2_client
        .authorize_security_group_ingress()
        .group_id(&group_id)
        .ip_permissions(
            IpPermission::builder()
                .ip_protocol("udp")
                .from_port(51820)
                .to_port(51820)
                .ip_ranges(IpRange::builder().cidr_ip("0.0.0.0/0").build())
                .ipv6_ranges(Ipv6Range::builder().cidr_ipv6("::/0").build())
                .build(),
        )
        .send()
        .await?;

    println!("Added SSH ingress rule to security group");

    Ok(group_id)
}

pub(super) async fn get_security_group_by_name(
    ec2_client: &Ec2Client,
    group_name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let filters = Filter::builder()
        .name("group-name")
        .values(group_name)
        .build();

    let resp = ec2_client
        .describe_security_groups()
        .filters(filters)
        .send()
        .await?;

    let group_id = resp
        .security_groups()
        .first()
        .and_then(|sg| sg.group_id())
        .map(|s| s.to_string());

    Ok(group_id)
}

pub(super) async fn create_vpc(
    ec2_client: &Ec2Client,
    cidr_block: &str,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
        .await?;

    let vpc_id = resp.vpc().and_then(|vpc| vpc.vpc_id()).unwrap();

    println!("Created VPC: {}", vpc_id);
    Ok(vpc_id.to_string())
}

pub(super) async fn get_vpc_by_name(
    ec2_client: &Ec2Client,
    name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let filter = Filter::builder()
        .name("tag:Name") // Tag-based filter
        .values(name)
        .build();

    let resp = ec2_client.describe_vpcs().filters(filter).send().await?;

    let vpc_id = resp
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(|id| id.to_string());

    Ok(vpc_id)
}

pub(super) async fn create_subnet(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    cidr_block: &str,
    ipv6_cidr_block: &str,
    az: &str,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tag_spec = TagSpecification::builder()
        .resource_type(ResourceType::Subnet)
        .tags(Tag::builder().key("Name").value(name).build())
        .build();

    let resp = ec2_client
        .create_subnet()
        .vpc_id(vpc_id)
        .cidr_block(cidr_block)
        .ipv6_cidr_block(ipv6_cidr_block)
        .availability_zone(az)
        .tag_specifications(tag_spec)
        .send()
        .await?;

    let subnet_id = resp.subnet().and_then(|subnet| subnet.subnet_id()).unwrap();

    println!("Created Subnet: {}", subnet_id);
    Ok(subnet_id.to_string())
}

pub(super) async fn list_availability_zones(
    ec2_client: &Ec2Client,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let resp = ec2_client.describe_availability_zones().send().await?;

    let azs = resp
        .availability_zones()
        .iter()
        .filter_map(|az| az.zone_name())
        .map(|az| az.to_string())
        .collect();

    Ok(azs)
}

pub(super) async fn get_vpc_ipv6_block(
    ec2_client: &Ec2Client,
    vpc_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let resp = ec2_client.describe_vpcs().vpc_ids(vpc_id).send().await?;

    let cidr = resp
        .vpcs()
        .iter()
        .flat_map(|vpc| vpc.ipv6_cidr_block_association_set())
        .filter_map(|assoc| assoc.ipv6_cidr_block())
        .next()
        .ok_or("No IPv6 CIDR block associated with VPC")?;

    Ok(cidr.to_string())
}

pub(super) fn carve_ipv6_subnet(
    base_cidr: &str,
    index: u8,
) -> Result<String, Box<dyn std::error::Error>> {
    let (base_ip, _prefix) = base_cidr.split_once('/').ok_or("Invalid IPv6 CIDR")?;
    let mut bytes = Ipv6Addr::from_str(base_ip)?.octets();

    // Increment the 8 bits after the /56 (byte 7)
    bytes[7] = index;

    let subnet = Ipv6Addr::from(bytes);
    Ok(format!("{}/64", subnet))
}

pub(super) async fn create_and_attach_igw(
    ec2: &Ec2Client,
    vpc_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let igw = ec2.create_internet_gateway().send().await?;
    let igw_id = igw
        .internet_gateway()
        .unwrap()
        .internet_gateway_id()
        .unwrap();

    ec2.attach_internet_gateway()
        .internet_gateway_id(igw_id)
        .vpc_id(vpc_id)
        .send()
        .await?;

    println!("🌐 Internet Gateway {igw_id} attached to VPC {vpc_id}");
    Ok(igw_id.to_string())
}

pub(super) async fn add_igw_routes_to_table(
    ec2: &Ec2Client,
    route_table_id: &str,
    igw_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // IPv4 default route
    let _ = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_cidr_block("0.0.0.0/0")
        .gateway_id(igw_id)
        .send()
        .await;

    // IPv6 default route
    let _ = ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_ipv6_cidr_block("::/0")
        .gateway_id(igw_id)
        .send()
        .await;

    println!("✅ Added default routes to route table: {}", route_table_id);
    Ok(())
}

pub(super) async fn enable_auto_ip_assign(
    ec2: &Ec2Client,
    subnet_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ec2.modify_subnet_attribute()
        .subnet_id(subnet_id)
        .map_public_ip_on_launch(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await?;

    ec2.modify_subnet_attribute()
        .subnet_id(subnet_id)
        .assign_ipv6_address_on_creation(AttributeBooleanValue::builder().value(true).build())
        .send()
        .await?;

    Ok(())
}

pub(super) async fn find_main_route_table(
    ec2: &Ec2Client,
    vpc_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let filters = Filter::builder().name("vpc-id").values(vpc_id).build();

    let resp = ec2.describe_route_tables().filters(filters).send().await?;

    let rt_id = resp
        .route_tables()
        .iter()
        .find(|rt| {
            rt.associations()
                .iter()
                .any(|assoc| assoc.main().unwrap_or(false))
        })
        .and_then(|rt| rt.route_table_id())
        .ok_or("No main route table found")?;

    Ok(rt_id.to_string())
}

pub async fn get_subnets_in_vpc(
    ec2_client: &Ec2Client,
    vpc_id: &str,
) -> Result<Vec<Subnet>, Box<dyn std::error::Error>> {
    let resp = ec2_client
        .describe_subnets()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("vpc-id")
                .values(vpc_id)
                .build(),
        )
        .send()
        .await?;

    Ok(resp.subnets().to_vec())
}

pub async fn tag_resource_with_name(
    ec2_client: &Ec2Client,
    resource_id: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tag = Tag::builder().key("Name").value(name).build();

    ec2_client
        .create_tags()
        .resources(resource_id)
        .tags(tag)
        .send()
        .await?;

    Ok(())
}
