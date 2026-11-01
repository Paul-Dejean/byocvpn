use std::collections::HashMap;

use byocvpn_core::error::Result;
use log::{debug, info};
use serde::de::IgnoredAny;

use crate::arm_resource::{API_VERSION_NETWORK, ArmResource, IpVersion, build_resource_group_name};
use crate::client::AzureClient;
use crate::models::{
    IpConfiguration, IpConfigurationProperties, NicProperties, NicRequest, PublicIpListResponse,
    PublicIpProperties, PublicIpRequest, PublicIpResponse, PublicIpSku, ResourceReference,
    byocvpn_tags,
};
use crate::resource_operations::{delete_resource, get_existing, put_and_wait};

pub async fn ensure_public_ip(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> Result<String> {
    let resource = ArmResource::public_ip(&build_resource_group_name(location), vm_name, version);
    if get_existing::<IgnoredAny>(client, &resource)
        .await?
        .is_some()
    {
        debug!("[Azure] {resource} already exists.");
        return Ok(client.build_resource_id(&resource));
    }

    info!("[Azure] Creating {resource}...");
    let body = PublicIpRequest {
        location: location.to_string(),
        sku: PublicIpSku {
            name: "Standard".to_string(),
        },
        tags: byocvpn_tags(),
        properties: PublicIpProperties {
            public_ip_allocation_method: "Static".to_string(),
            public_ip_address_version: version.get_address_version().to_string(),
        },
    };
    put_and_wait(client, &resource, &body)
        .await
        .map_err(|error| error.with_context("creating public ip address"))?;
    info!("[Azure] {resource} created.");
    Ok(client.build_resource_id(&resource))
}

pub async fn get_public_ip_address(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> Result<String> {
    let resource = ArmResource::public_ip(&build_resource_group_name(location), vm_name, version);
    let public_ip: PublicIpResponse = client.get(&client.build_resource_url(&resource)).await?;
    Ok(public_ip
        .properties
        .and_then(|properties| properties.ip_address)
        .unwrap_or_default())
}

pub struct PublicIpAddresses {
    addresses_by_resource_name: HashMap<String, String>,
}

impl PublicIpAddresses {
    pub fn get_address(&self, resource_group: &str, vm_name: &str, version: IpVersion) -> String {
        let resource = ArmResource::public_ip(resource_group, vm_name, version);
        self.addresses_by_resource_name
            .get(resource.get_name())
            .cloned()
            .unwrap_or_default()
    }
}

pub async fn list_public_ip_addresses(
    client: &AzureClient,
    resource_group: &str,
) -> Result<PublicIpAddresses> {
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{resource_group}/providers/Microsoft.Network/publicIPAddresses"
    ));
    let url = AzureClient::build_arm_url(&path, API_VERSION_NETWORK);
    let response: PublicIpListResponse = client
        .get(&url)
        .await
        .map_err(|error| error.with_context(format!("listing public ips in {resource_group}")))?;

    let addresses_by_resource_name = response
        .value
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let address = item.properties?.ip_address?;
            Some((item.name?, address))
        })
        .collect();
    Ok(PublicIpAddresses {
        addresses_by_resource_name,
    })
}

pub async fn create_nic(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    subnet_id: &str,
    public_ipv4_id: &str,
    public_ipv6_id: &str,
    nsg_id: &str,
) -> Result<String> {
    let resource = ArmResource::network_interface(&build_resource_group_name(location), vm_name);

    let body = NicRequest {
        location: location.to_string(),
        tags: byocvpn_tags(),
        properties: NicProperties {
            network_security_group: ResourceReference {
                id: nsg_id.to_string(),
            },
            ip_configurations: vec![
                build_ip_configuration("ipconfig1", IpVersion::V4, subnet_id, public_ipv4_id, true),
                build_ip_configuration(
                    "ipconfig2",
                    IpVersion::V6,
                    subnet_id,
                    public_ipv6_id,
                    false,
                ),
            ],
        },
    };

    info!("[Azure] Creating {resource} in {location}...");
    put_and_wait(client, &resource, &body)
        .await
        .map_err(|error| error.with_context("creating network interface"))?;
    info!("[Azure] {resource} created in {location}.");
    Ok(client.build_resource_id(&resource))
}

pub async fn cleanup_vm_resources(client: &AzureClient, resource_group: &str, vm_name: &str) {
    info!(
        "[Azure] Cleaning up network resources for VM '{vm_name}' in resource group '{resource_group}'..."
    );
    let network_interface = ArmResource::network_interface(resource_group, vm_name);
    let public_ipv4 = ArmResource::public_ip(resource_group, vm_name, IpVersion::V4);
    let public_ipv6 = ArmResource::public_ip(resource_group, vm_name, IpVersion::V6);
    delete_resource(client, &network_interface).await;
    tokio::join!(
        delete_resource(client, &public_ipv4),
        delete_resource(client, &public_ipv6),
    );
}

fn build_ip_configuration(
    name: &str,
    version: IpVersion,
    subnet_id: &str,
    public_ip_id: &str,
    is_primary: bool,
) -> IpConfiguration {
    IpConfiguration {
        name: name.to_string(),
        properties: IpConfigurationProperties {
            private_ip_allocation_method: "Dynamic".to_string(),
            private_ip_address_version: version.get_address_version().to_string(),
            subnet: ResourceReference {
                id: subnet_id.to_string(),
            },
            public_ip_address: ResourceReference {
                id: public_ip_id.to_string(),
            },
            primary: is_primary.then_some(true),
        },
    }
}
