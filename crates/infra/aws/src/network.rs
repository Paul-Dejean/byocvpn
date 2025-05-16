use aws_sdk_ec2::{
    Client as Ec2Client,
    types::{Filter, IpPermission, IpRange, Ipv6Range, ResourceType, Tag, TagSpecification},
};

pub(super) async fn create_security_group(
    ec2_client: &Ec2Client,
    group_name: &str,
    description: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Set region provider

    // 1. Create security group
    let create_resp = ec2_client
        .create_security_group()
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

pub async fn get_byocvpn_sg_id(
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

pub async fn create_vpc(
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

pub async fn create_subnet(
    ec2_client: &Ec2Client,
    vpc_id: &str,
    cidr_block: &str,
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
        .availability_zone(az)
        .tag_specifications(tag_spec)
        .send()
        .await?;

    let subnet_id = resp.subnet().and_then(|subnet| subnet.subnet_id()).unwrap();

    println!("Created Subnet: {}", subnet_id);
    Ok(subnet_id.to_string())
}
