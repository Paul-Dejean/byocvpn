use log::{debug, info};

use crate::{
    arm_resource::ArmResource,
    client::AzureClient,
    error::Result,
    models::{ResourceReference, SubnetRequest, SubnetRequestProperties, SubnetResponse},
    resource_operations::{get_existing, put_and_wait},
};

const SUBNET_CIDR: &str = "10.0.0.0/24";
const SUBNET_IPV6_CIDR: &str = "fd00::/64";

pub async fn ensure_subnet(
    client: &AzureClient,
    location: &str,
    network_security_group_id: &str,
) -> Result<String> {
    let resource = ArmResource::subnet(location);
    let existing = get_existing::<SubnetResponse>(client, &resource).await?;

    let existing_prefixes: Vec<String> = existing
        .as_ref()
        .and_then(|subnet| subnet.properties.as_ref())
        .and_then(|properties| properties.address_prefixes.clone())
        .unwrap_or_default();
    let has_ipv6 = existing_prefixes.iter().any(|prefix| prefix.contains(':'));
    let has_desired_network_security_group = existing
        .as_ref()
        .and_then(|subnet| subnet.properties.as_ref())
        .and_then(|properties| properties.network_security_group.as_ref())
        .is_some_and(|reference| reference.id.eq_ignore_ascii_case(network_security_group_id));

    if existing.is_some() && has_ipv6 && has_desired_network_security_group {
        debug!("[Azure] {resource} already exists in {location} and is up to date.");
        return Ok(client.build_resource_id(&resource));
    }

    let address_prefixes = if has_ipv6 {
        existing_prefixes
    } else {
        vec![SUBNET_CIDR.to_string(), SUBNET_IPV6_CIDR.to_string()]
    };
    if existing.is_some() {
        info!(
            "[Azure] {resource} has drifted (ipv6={has_ipv6}, network_security_group={has_desired_network_security_group}), updating..."
        );
    } else {
        info!("[Azure] Creating {resource} ({address_prefixes:?}) in {location}...");
    }

    let body = SubnetRequest {
        properties: SubnetRequestProperties {
            address_prefixes,
            network_security_group: ResourceReference {
                id: network_security_group_id.to_string(),
            },
        },
    };
    put_and_wait(client, &resource, &body).await?;
    info!("[Azure] {resource} ready in {location}.");
    Ok(client.build_resource_id(&resource))
}
