use byocvpn_core::error::Result;
use log::{debug, info};
use serde::de::IgnoredAny;

use crate::arm_resource::ArmResource;
use crate::client::AzureClient;
use crate::models::{AddressSpace, VnetProperties, VnetRequest, byocvpn_tags};
use crate::resource_operations::{get_existing, put_and_wait};

const VNET_CIDR: &str = "10.0.0.0/16";
const VNET_IPV6_CIDR: &str = "fd00::/48";

pub async fn ensure_vnet(client: &AzureClient, location: &str) -> Result<String> {
    let resource = ArmResource::virtual_network(location);
    if get_existing::<IgnoredAny>(client, &resource)
        .await?
        .is_some()
    {
        debug!("[Azure] {resource} already exists in {location}.");
        return Ok(client.build_resource_id(&resource));
    }

    info!("[Azure] Creating {resource} in {location}...");
    let body = VnetRequest {
        location: location.to_string(),
        tags: byocvpn_tags(),
        properties: VnetProperties {
            address_space: AddressSpace {
                address_prefixes: vec![VNET_CIDR.to_string(), VNET_IPV6_CIDR.to_string()],
            },
        },
    };
    put_and_wait(client, &resource, &body)
        .await
        .map_err(|error| error.with_context("creating virtual network"))?;
    info!("[Azure] {resource} created in {location}.");
    Ok(client.build_resource_id(&resource))
}
