use log::info;

use crate::{
    client::OciClient,
    error::Result,
    models::{
        CreateSubnetRequest, SubnetResponse, UpdateSubnetSecurityListRequest, build_managed_tags,
    },
    network::vcn::Vcn,
};

const SUBNET_DISPLAY_NAME: &str = "byocvpn-subnet";
const SUBNET_CIDR: &str = "10.0.0.0/24";
const TERMINATED_LIFECYCLE_STATES: [&str; 2] = ["TERMINATING", "TERMINATED"];

pub async fn ensure_subnet(
    client: &OciClient,
    vcn: &Vcn,
    security_list_id: &str,
) -> Result<String> {
    if let Some(existing) = get_subnet(client, vcn).await? {
        let attached_security_lists = existing.security_list_ids.unwrap_or_default();
        if !attached_security_lists
            .iter()
            .any(|id| id == security_list_id)
        {
            attach_security_list(client, &existing.id, security_list_id).await?;
        }
        return Ok(existing.id);
    }
    create_subnet(client, vcn, security_list_id).await
}

async fn get_subnet(client: &OciClient, vcn: &Vcn) -> Result<Option<SubnetResponse>> {
    let url = client.build_core_url(&format!(
        "/subnets?compartmentId={}&vcnId={}&displayName={SUBNET_DISPLAY_NAME}",
        client.get_compartment_ocid(),
        vcn.id
    ));
    let subnets: Vec<SubnetResponse> = client.get(&url).await?;
    Ok(subnets
        .into_iter()
        .find(|subnet| !TERMINATED_LIFECYCLE_STATES.contains(&subnet.lifecycle_state.as_str())))
}

async fn create_subnet(client: &OciClient, vcn: &Vcn, security_list_id: &str) -> Result<String> {
    let body = CreateSubnetRequest {
        compartment_id: client.get_compartment_ocid().to_string(),
        vcn_id: vcn.id.clone(),
        display_name: SUBNET_DISPLAY_NAME.to_string(),
        cidr_block: SUBNET_CIDR.to_string(),
        security_list_ids: vec![security_list_id.to_string()],
        route_table_id: vcn.default_route_table_id.clone(),
        freeform_tags: build_managed_tags(),
        ipv6_cidr_blocks: vcn
            .has_ipv6()
            .then(|| vec![derive_first_ipv6_subnet(&vcn.ipv6_prefix)]),
    };
    let subnet: SubnetResponse = client
        .post(&client.build_core_url("/subnets"), &body)
        .await?;
    info!("[OCI] Created subnet {}", subnet.id);
    Ok(subnet.id)
}

async fn attach_security_list(
    client: &OciClient,
    subnet_id: &str,
    security_list_id: &str,
) -> Result<()> {
    let url = client.build_core_url(&format!("/subnets/{subnet_id}"));
    let body = UpdateSubnetSecurityListRequest {
        security_list_ids: vec![security_list_id.to_string()],
    };
    client.put(&url, &body).await?;
    info!("[OCI] Security list {security_list_id} attached to subnet {subnet_id}");
    Ok(())
}

fn derive_first_ipv6_subnet(vcn_ipv6_prefix: &str) -> String {
    let network_address = vcn_ipv6_prefix
        .rsplit_once('/')
        .map_or(vcn_ipv6_prefix, |(address, _prefix_length)| address);
    format!("{network_address}/64")
}
