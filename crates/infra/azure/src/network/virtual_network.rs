use byocvpn_core::managed_resources::build_managed_tags;
use serde::de::IgnoredAny;
use tracing::{debug, info};

use crate::{
    arm_resource::ArmResource,
    client::AzureClient,
    error::Result,
    models::{AddressSpace, VnetProperties, VnetRequest},
    resource_operations::{get_existing, put_and_wait},
};

const VNET_CIDR: &str = "10.0.0.0/16";
const VNET_IPV6_CIDR: &str = "fd00::/48";

pub async fn ensure_virtual_network(client: &AzureClient, location: &str) -> Result<String> {
    let resource = ArmResource::virtual_network(location);
    if get_existing::<IgnoredAny>(client, &resource)
        .await?
        .is_some()
    {
        debug!(%resource, %location, "already exists");
        return Ok(client.build_resource_id(&resource));
    }

    debug!(%resource, %location, "creating");
    let body = VnetRequest {
        location: location.to_string(),
        tags: build_managed_tags(),
        properties: VnetProperties {
            address_space: AddressSpace {
                address_prefixes: vec![VNET_CIDR.to_string(), VNET_IPV6_CIDR.to_string()],
            },
        },
    };
    put_and_wait(client, &resource, &body).await?;
    info!(%resource, %location, "created");
    Ok(client.build_resource_id(&resource))
}
