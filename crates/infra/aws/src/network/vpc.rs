use aws_sdk_ec2::types::{Filter, ResourceType, Tag, TagSpecification, Vpc};
use log::info;

use crate::{
    client::AwsClient,
    error::{AwsError, Result},
};

const VPC_NAME: &str = "byocvpn-vpc";
const VPC_CIDR_BLOCK: &str = "10.0.0.0/16";

pub async fn ensure_vpc(client: &AwsClient) -> Result<String> {
    if let Some(vpc_id) = get_vpc_by_name(client).await? {
        return Ok(vpc_id);
    }
    create_vpc(client).await
}

pub async fn get_vpc_ipv6_block(client: &AwsClient, vpc_id: &str) -> Result<String> {
    let response = client.ec2.describe_vpcs().vpc_ids(vpc_id).send().await?;

    response
        .vpcs()
        .iter()
        .flat_map(Vpc::ipv6_cidr_block_association_set)
        .find_map(|association| association.ipv6_cidr_block())
        .map(ToString::to_string)
        .ok_or_else(|| AwsError::MissingVpcIpv6Cidr {
            vpc_id: vpc_id.to_string(),
        })
}

async fn get_vpc_by_name(client: &AwsClient) -> Result<Option<String>> {
    let response = client
        .ec2
        .describe_vpcs()
        .filters(Filter::builder().name("tag:Name").values(VPC_NAME).build())
        .send()
        .await?;

    Ok(response
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(ToString::to_string))
}

async fn create_vpc(client: &AwsClient) -> Result<String> {
    let tag_specification = TagSpecification::builder()
        .resource_type(ResourceType::Vpc)
        .tags(Tag::builder().key("Name").value(VPC_NAME).build())
        .build();

    let response = client
        .ec2
        .create_vpc()
        .cidr_block(VPC_CIDR_BLOCK)
        .amazon_provided_ipv6_cidr_block(true)
        .tag_specifications(tag_specification)
        .send()
        .await?;

    let vpc_id =
        response
            .vpc()
            .and_then(|vpc| vpc.vpc_id())
            .ok_or(AwsError::MissingResponseField {
                operation: "ec2:CreateVpc",
                field: "vpcId",
            })?;
    info!("[AWS] Created VPC {vpc_id}");
    Ok(vpc_id.to_string())
}
