use aws_sdk_ec2::types::{Filter, RouteTable, Tag};
use byocvpn_core::managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR};
use log::info;

use crate::{
    client::AwsClient,
    error::{AwsError, Result},
};

const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";
const MAIN_ROUTE_TABLE_NAME: &str = "byocvpn-main-route-table";
const ROUTE_ALREADY_EXISTS_CODE: &str = "RouteAlreadyExists";

pub async fn ensure_internet_gateway(client: &AwsClient, vpc_id: &str) -> Result<String> {
    let route_table_id = find_main_route_table(client, vpc_id).await?;
    if let Some(internet_gateway_id) = get_internet_gateway(client, vpc_id).await? {
        if !has_default_routes(client, &route_table_id, &internet_gateway_id).await? {
            info!("[AWS] Internet gateway routes missing or modified, reconfiguring...");
            add_default_routes(client, &route_table_id, &internet_gateway_id).await?;
        }
        return Ok(internet_gateway_id);
    }
    let internet_gateway_id = create_internet_gateway(client, vpc_id).await?;
    tag_resource_with_name(client, &route_table_id, MAIN_ROUTE_TABLE_NAME).await?;
    tag_resource_with_name(client, &internet_gateway_id, INTERNET_GATEWAY_NAME).await?;
    add_default_routes(client, &route_table_id, &internet_gateway_id).await?;
    Ok(internet_gateway_id)
}

async fn find_main_route_table(client: &AwsClient, vpc_id: &str) -> Result<String> {
    let response = client
        .ec2
        .describe_route_tables()
        .filters(Filter::builder().name("vpc-id").values(vpc_id).build())
        .send()
        .await?;

    response
        .route_tables()
        .iter()
        .find(|route_table| {
            route_table
                .associations()
                .iter()
                .any(|association| association.main().unwrap_or(false))
        })
        .and_then(|route_table| route_table.route_table_id())
        .map(ToString::to_string)
        .ok_or_else(|| AwsError::MissingMainRouteTable {
            vpc_id: vpc_id.to_string(),
        })
}

async fn get_internet_gateway(client: &AwsClient, vpc_id: &str) -> Result<Option<String>> {
    let response = client
        .ec2
        .describe_internet_gateways()
        .filters(
            Filter::builder()
                .name("attachment.vpc-id")
                .values(vpc_id)
                .build(),
        )
        .send()
        .await?;

    Ok(response
        .internet_gateways()
        .first()
        .and_then(|internet_gateway| internet_gateway.internet_gateway_id())
        .map(ToString::to_string))
}

async fn create_internet_gateway(client: &AwsClient, vpc_id: &str) -> Result<String> {
    let response = client.ec2.create_internet_gateway().send().await?;
    let internet_gateway_id = response
        .internet_gateway()
        .and_then(|internet_gateway| internet_gateway.internet_gateway_id())
        .ok_or(AwsError::MissingResponseField {
            operation: "ec2:CreateInternetGateway",
            field: "internetGatewayId",
        })?;

    client
        .ec2
        .attach_internet_gateway()
        .internet_gateway_id(internet_gateway_id)
        .vpc_id(vpc_id)
        .send()
        .await?;

    info!("[AWS] Internet gateway {internet_gateway_id} attached to VPC {vpc_id}");
    Ok(internet_gateway_id.to_string())
}

async fn has_default_routes(
    client: &AwsClient,
    route_table_id: &str,
    internet_gateway_id: &str,
) -> Result<bool> {
    let response = client
        .ec2
        .describe_route_tables()
        .filters(
            Filter::builder()
                .name("route-table-id")
                .values(route_table_id)
                .build(),
        )
        .send()
        .await?;

    let routes = response
        .route_tables()
        .first()
        .map(RouteTable::routes)
        .unwrap_or_default();
    let has_ipv4_route = routes.iter().any(|route| {
        route.gateway_id() == Some(internet_gateway_id)
            && route.destination_cidr_block() == Some(ANY_IPV4_CIDR)
    });
    let has_ipv6_route = routes.iter().any(|route| {
        route.gateway_id() == Some(internet_gateway_id)
            && route.destination_ipv6_cidr_block() == Some(ANY_IPV6_CIDR)
    });
    Ok(has_ipv4_route && has_ipv6_route)
}

async fn add_default_routes(
    client: &AwsClient,
    route_table_id: &str,
    internet_gateway_id: &str,
) -> Result<()> {
    let ipv4_outcome = client
        .ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_cidr_block(ANY_IPV4_CIDR)
        .gateway_id(internet_gateway_id)
        .send()
        .await;
    if let Err(sdk_error) = ipv4_outcome {
        tolerate_existing_route(AwsError::from(sdk_error))?;
    }

    let ipv6_outcome = client
        .ec2
        .create_route()
        .route_table_id(route_table_id)
        .destination_ipv6_cidr_block(ANY_IPV6_CIDR)
        .gateway_id(internet_gateway_id)
        .send()
        .await;
    if let Err(sdk_error) = ipv6_outcome {
        tolerate_existing_route(AwsError::from(sdk_error))?;
    }

    info!("[AWS] Added default routes to route table {route_table_id}");
    Ok(())
}

fn tolerate_existing_route(error: AwsError) -> Result<()> {
    if error.has_code(ROUTE_ALREADY_EXISTS_CODE) {
        Ok(())
    } else {
        Err(error)
    }
}

async fn tag_resource_with_name(client: &AwsClient, resource_id: &str, name: &str) -> Result<()> {
    client
        .ec2
        .create_tags()
        .resources(resource_id)
        .tags(Tag::builder().key("Name").value(name).build())
        .send()
        .await?;
    Ok(())
}
