use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    managed_resources::{ANY_IPV4_CIDR, ANY_IPV6_CIDR},
};
use log::info;

use crate::{
    client::OciClient,
    models::{
        CreateInternetGatewayRequest, InternetGatewayResponse, RouteRule, RouteTableResponse,
        UpdateRouteTableRequest, build_managed_tags,
    },
    network::vcn::Vcn,
};

const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";
const CIDR_BLOCK_DESTINATION_TYPE: &str = "CIDR_BLOCK";

pub async fn ensure_internet_gateway(client: &OciClient, vcn: &Vcn) -> Result<String> {
    let internet_gateway_id = match get_internet_gateway(client, vcn).await? {
        Some(existing_id) => existing_id,
        None => create_internet_gateway(client, vcn).await?,
    };
    ensure_default_routes(client, vcn, &internet_gateway_id).await?;
    Ok(internet_gateway_id)
}

async fn get_internet_gateway(client: &OciClient, vcn: &Vcn) -> Result<Option<String>> {
    let url = client.build_core_url(&format!(
        "/internetGateways?compartmentId={}&vcnId={}",
        client.get_compartment_ocid(),
        vcn.id
    ));
    let gateways: Vec<InternetGatewayResponse> = client.get(&url).await?;
    Ok(gateways.into_iter().next().map(|gateway| gateway.id))
}

async fn create_internet_gateway(client: &OciClient, vcn: &Vcn) -> Result<String> {
    let body = CreateInternetGatewayRequest {
        compartment_id: client.get_compartment_ocid().to_string(),
        vcn_id: vcn.id.clone(),
        display_name: INTERNET_GATEWAY_NAME.to_string(),
        is_enabled: true,
        freeform_tags: build_managed_tags(),
    };
    let gateway: InternetGatewayResponse = client
        .post(&client.build_core_url("/internetGateways"), &body)
        .await
        .map_err(
            |error| NetworkProvisioningError::InternetGatewayOperationFailed {
                reason: error.to_string(),
            },
        )?;
    info!("[OCI] Created internet gateway {}", gateway.id);
    Ok(gateway.id)
}

async fn ensure_default_routes(
    client: &OciClient,
    vcn: &Vcn,
    internet_gateway_id: &str,
) -> Result<()> {
    let url = client.build_core_url(&format!("/routeTables/{}", vcn.default_route_table_id));
    let current: RouteTableResponse = client.get(&url).await?;
    let mut route_rules = current.route_rules;

    let mut wanted_destinations = vec![ANY_IPV4_CIDR];
    if vcn.has_ipv6() {
        wanted_destinations.push(ANY_IPV6_CIDR);
    }
    for destination in wanted_destinations {
        if !route_rules
            .iter()
            .any(|rule| rule.destination == destination)
        {
            route_rules.push(RouteRule {
                network_entity_id: internet_gateway_id.to_string(),
                destination: destination.to_string(),
                destination_type: CIDR_BLOCK_DESTINATION_TYPE.to_string(),
            });
        }
    }

    client
        .put(&url, &UpdateRouteTableRequest { route_rules })
        .await
        .map_err(|error| error.with_context("route table operation failed"))?;
    info!(
        "[OCI] Default routes ensured on route table {}",
        vcn.default_route_table_id
    );
    Ok(())
}
