use byocvpn_core::error::{NetworkProvisioningError, Result};
use log::*;
use serde::Serialize;
use serde_json::json;
use tokio::time::{Duration, sleep};

use crate::client::AzureClient;

const API_VERSION_RESOURCE_GROUPS: &str = "2021-04-01";
pub(crate) const API_VERSION_NETWORK: &str = "2024-05-01";
const API_VERSION_LOCATIONS: &str = "2022-12-01";

const VPC_NAME: &str = "byocvpn-vnet";
const SUBNET_NAME: &str = "byocvpn-subnet";
const NSG_NAME: &str = "byocvpn-nsg";

// ── CIDR helpers ─────────────────────────────────────────────────────────────

fn compute_location_index(location: &str) -> u8 {
    const LOCATIONS: &[(&str, u8)] = &[
        ("eastus", 50),
        ("eastus2", 51),
        ("westus", 52),
        ("westus2", 53),
        ("westus3", 54),
        ("centralus", 55),
        ("northcentralus", 56),
        ("southcentralus", 57),
        ("westcentralus", 58),
        ("canadacentral", 59),
        ("canadaeast", 60),
        ("brazilsouth", 61),
        ("brazilsoutheast", 62),
        ("northeurope", 63),
        ("westeurope", 64),
        ("uksouth", 65),
        ("ukwest", 66),
        ("francecentral", 67),
        ("francesouth", 68),
        ("germanywestcentral", 69),
        ("germanynorth", 70),
        ("norwayeast", 71),
        ("norwaywest", 72),
        ("swedencentral", 73),
        ("switzerlandnorth", 74),
        ("switzerlandwest", 75),
        ("polandcentral", 76),
        ("italynorth", 77),
        ("spaincentral", 78),
        ("eastasia", 79),
        ("southeastasia", 80),
        ("australiaeast", 81),
        ("australiasoutheast", 82),
        ("australiacentral", 83),
        ("australiacentral2", 84),
        ("japaneast", 85),
        ("japanwest", 86),
        ("koreacentral", 87),
        ("koreasouth", 88),
        ("centralindia", 89),
        ("southindia", 90),
        ("westindia", 91),
        ("jioindiacentral", 92),
        ("jioindiawest", 93),
        ("newzealandnorth", 94),
        ("uaenorth", 95),
        ("uaecentral", 96),
        ("qatarcentral", 97),
        ("israelcentral", 98),
        ("southafricanorth", 99),
        ("southafricawest", 100),
        ("mexicocentral", 101),
        ("chilecentral", 102),
        ("eastus2euap", 103),
        ("westus2euap", 104),
    ];

    if let Some((_, index)) = LOCATIONS.iter().find(|(name, _)| *name == location) {
        return *index;
    }

    let hash: u8 = location
        .bytes()
        .enumerate()
        .fold(0u32, |acc, (i, b)| {
            acc.wrapping_add((b as u32).wrapping_mul((i as u32).wrapping_add(1)))
        })
        .wrapping_rem(50) as u8;
    150 + hash
}

fn build_vnet_cidr_for_location(location: &str) -> String {
    format!("10.{}.0.0/16", compute_location_index(location))
}

fn build_subnet_cidr_for_location(location: &str) -> String {
    format!("10.{}.0.0/24", compute_location_index(location))
}

fn build_vnet_ipv6_cidr_for_location(location: &str) -> String {
    format!("fd{:02x}::/48", compute_location_index(location))
}

fn build_subnet_ipv6_cidr_for_location(location: &str) -> String {
    format!("fd{:02x}::/64", compute_location_index(location))
}

// ── Regions ──────────────────────────────────────────────────────────────────

pub async fn list_regions(client: &AzureClient) -> Result<Vec<(String, String)>> {
    let path = client.build_subscription_path("/locations");
    let url = client.build_arm_url(&path, API_VERSION_LOCATIONS);

    let response =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;

    let continent_prefixes: &[(&str, &str)] = &[
        ("eastus", "United States"),
        ("westus", "United States"),
        ("centralus", "United States"),
        ("northcentralus", "United States"),
        ("southcentralus", "United States"),
        ("westcentralus", "United States"),
        ("canadacentral", "Canada"),
        ("canadaeast", "Canada"),
        ("brazilsouth", "South America"),
        ("brazilsoutheast", "South America"),
        ("northeurope", "Europe"),
        ("westeurope", "Europe"),
        ("uksouth", "Europe"),
        ("ukwest", "Europe"),
        ("francecentral", "Europe"),
        ("francesouth", "Europe"),
        ("germanywestcentral", "Europe"),
        ("germanynorth", "Europe"),
        ("norwayeast", "Europe"),
        ("norwaywest", "Europe"),
        ("swedencentral", "Europe"),
        ("switzerlandnorth", "Europe"),
        ("switzerlandwest", "Europe"),
        ("polandcentral", "Europe"),
        ("italynorth", "Europe"),
        ("spaincentral", "Europe"),
        ("eastasia", "Asia Pacific"),
        ("southeastasia", "Asia Pacific"),
        ("australiaeast", "Asia Pacific"),
        ("australiasoutheast", "Asia Pacific"),
        ("australiacentral", "Asia Pacific"),
        ("japaneast", "Asia Pacific"),
        ("japanwest", "Asia Pacific"),
        ("koreacentral", "Asia Pacific"),
        ("koreasouth", "Asia Pacific"),
        ("centralindia", "Asia Pacific"),
        ("southindia", "Asia Pacific"),
        ("westindia", "Asia Pacific"),
        ("jioindiacentral", "Asia Pacific"),
        ("jioindiawest", "Asia Pacific"),
        ("newzealandnorth", "Asia Pacific"),
        ("uaenorth", "Middle East"),
        ("uaecentral", "Middle East"),
        ("qatarcentral", "Middle East"),
        ("israelcentral", "Middle East"),
        ("southafricanorth", "Africa"),
        ("southafricawest", "Africa"),
        ("mexicocentral", "North America"),
        ("chilecentral", "South America"),
    ];

    let regions = response["value"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|location| {
            let name = location["name"].as_str()?.to_string();

            let category = location["metadata"]["regionCategory"].as_str()?;
            if category != "Recommended" {
                return None;
            }
            let continent = continent_prefixes
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix))
                .map(|(_, continent)| continent.to_string())
                .unwrap_or_else(|| "Other".to_string());
            Some((name, continent))
        })
        .collect();

    Ok(regions)
}

// ── Resource group ────────────────────────────────────────────────────────────

pub fn build_resource_group_name(location: &str) -> String {
    format!("byocvpn-{}", location)
}

pub struct RegionNetworkIds {
    pub subnet_id: String,
    pub nsg_id: String,
}

pub async fn ensure_region_networking(
    client: &AzureClient,
    location: &str,
) -> Result<RegionNetworkIds> {
    if get_resource_group_by_location(client, location)
        .await?
        .is_none()
    {
        create_resource_group(client, location).await?;
    }
    let nsg_id = ensure_nsg(client, location).await?;
    if get_vnet(client, location).await?.is_none() {
        create_vnet(client, location).await?;
    }
    let subnet_id = match get_subnet(client, location).await? {
        Some(id) => id,
        None => create_subnet(client, location, &nsg_id).await?,
    };
    Ok(RegionNetworkIds { subnet_id, nsg_id })
}

async fn put_with_provider_retry<B: Serialize>(
    client: &AzureClient,
    url: &str,
    body: &B,
    resource_label: &str,
) -> Result<Option<String>> {
    let max_attempts = 120u32;
    for attempt in 1..=max_attempts {
        match client.put(url, body).await {
            Ok(op_url) => return Ok(op_url),
            Err(error) => {
                let error_message = error.to_string();
                if error_message.contains("MissingSubscriptionRegistration") {
                    warn!(
                        "[Azure] {} — 409 response: {}",
                        resource_label, error_message
                    );
                    info!(
                        "[Azure] {} — re-registering and retrying in 15 s (attempt {}/{}, {}s elapsed)...",
                        resource_label,
                        attempt,
                        max_attempts,
                        attempt * 15
                    );

                    let register_path =
                        client.build_subscription_path("/providers/Microsoft.Network/register");
                    let register_url =
                        client.build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
                    match client.post(&register_url, &serde_json::Value::Null).await {
                        Ok(body) => debug!("[Azure] Re-registration POST succeeded: {}", body),
                        Err(reg_error) => {
                            warn!("[Azure] Re-registration POST failed: {}", reg_error)
                        }
                    }
                    sleep(Duration::from_secs(15)).await;
                } else {
                    return Err(NetworkProvisioningError::NetworkQueryFailed {
                        reason: format!("{} PUT failed: {}", resource_label, error_message),
                    }
                    .into());
                }
            }
        }
    }
    Err(NetworkProvisioningError::CloudOperationTimedOut {
        operation: resource_label.to_string(),
    }
    .into())
}

pub async fn ensure_providers_registered(client: &AzureClient) -> Result<()> {
    const NAMESPACES: &[&str] = &[
        "Microsoft.Network",
        "Microsoft.Compute",
        "Microsoft.Resources",
    ];

    for namespace in NAMESPACES {
        let status_path = client.build_subscription_path(&format!("/providers/{}", namespace));
        let status_url = client.build_arm_url(&status_path, API_VERSION_RESOURCE_GROUPS);
        let state = client.get(&status_url).await.ok().and_then(|body| {
            body["registrationState"]
                .as_str()
                .map(|state_str| state_str.to_string())
        });

        if state.as_deref() == Some("Registered") {
            continue;
        }

        let register_path =
            client.build_subscription_path(&format!("/providers/{}/register", namespace));
        let register_url = client.build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
        info!("[Azure] Registering provider '{}'...", namespace);
        client
            .post(&register_url, &serde_json::Value::Null)
            .await
            .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
                step: namespace.to_string(),
                reason: error.to_string(),
            })?;

        for attempt in 1..=60u32 {
            sleep(Duration::from_secs(5)).await;
            let body = client.get(&status_url).await.map_err(|error| {
                NetworkProvisioningError::ProviderSetupFailed {
                    step: namespace.to_string(),
                    reason: error.to_string(),
                }
            })?;
            let registration_state = body["registrationState"].as_str().unwrap_or("Unknown");
            debug!(
                "[Azure] Provider '{}' registration state: {} (attempt {}/60)",
                namespace, registration_state, attempt
            );
            if registration_state == "Registered" {
                break;
            }
            if attempt == 60 {
                return Err(NetworkProvisioningError::CloudOperationTimedOut {
                    operation: format!("{} provider registration", namespace),
                }
                .into());
            }
        }
    }
    Ok(())
}

pub async fn get_resource_group_by_location(
    client: &AzureClient,
    location: &str,
) -> Result<Option<String>> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!("/resourceGroups/{}", resource_group));
    let url = client.build_arm_url(&path, API_VERSION_RESOURCE_GROUPS);

    match client.get(&url).await {
        Ok(_) => Ok(Some(resource_group)),
        Err(_) => Ok(None),
    }
}

pub async fn create_resource_group(client: &AzureClient, location: &str) -> Result<()> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!("/resourceGroups/{}", resource_group));
    let url = client.build_arm_url(&path, API_VERSION_RESOURCE_GROUPS);

    let body = json!({
        "location": location,
        "tags": { "created-by": "byocvpn" }
    });

    info!("[Azure] Creating resource group '{}'...", resource_group);
    let async_op_url = put_with_provider_retry(
        client,
        &url,
        &body,
        &format!("resource group '{}'", resource_group),
    )
    .await
    .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
        reason: format!("Failed to create resource group: {}", error),
    })?;

    if let Some(op_url) = async_op_url {
        debug!(
            "[Azure] Waiting for resource group '{}' to be ready...",
            resource_group
        );
        client.wait_for_async_operation(&op_url).await?;
    }

    info!("[Azure] Resource group '{}' created.", resource_group);
    Ok(())
}

pub async fn ensure_nsg(client: &AzureClient, location: &str) -> Result<String> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups/{}",
        resource_group, NSG_NAME
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    if let Ok(existing) = client.get(&url).await {
        return Ok(existing["id"].as_str().unwrap_or_default().to_string());
    }

    let body = json!({
        "location": location,
        "tags": { "created-by": "byocvpn" },
        "properties": {
            "securityRules": [
                {
                    "name": "AllowWireGuard",
                    "properties": {
                        "priority": 1000,
                        "protocol": "UDP",
                        "access": "Allow",
                        "direction": "Inbound",
                        "sourceAddressPrefix": "*",
                        "sourcePortRange": "*",
                        "destinationAddressPrefix": "*",
                        "destinationPortRange": "51820"
                    }
                },
                {
                    "name": "AllowHealthEndpoint",
                    "properties": {
                        "priority": 1001,
                        "protocol": "TCP",
                        "access": "Allow",
                        "direction": "Inbound",
                        "sourceAddressPrefix": "*",
                        "sourcePortRange": "*",
                        "destinationAddressPrefix": "*",
                        "destinationPortRange": "51820"
                    }
                }
            ]
        }
    });

    info!("[Azure] Creating NSG '{}' in {}...", NSG_NAME, location);
    let async_op_url = put_with_provider_retry(client, &url, &body, &format!("NSG '{}'", NSG_NAME))
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            },
        )?;

    if let Some(op_url) = async_op_url {
        debug!(
            "[Azure] Waiting for NSG '{}' to be provisioned...",
            NSG_NAME
        );
        client.wait_for_async_operation(&op_url).await?;
    }

    let nsg = client.get(&url).await?;
    info!("[Azure] NSG '{}' created in {}.", NSG_NAME, location);
    Ok(nsg["id"].as_str().unwrap_or_default().to_string())
}

pub async fn get_vnet(client: &AzureClient, location: &str) -> Result<Option<String>> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}",
        resource_group, VPC_NAME
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    match client.get(&url).await {
        Ok(body) => Ok(Some(body["id"].as_str().unwrap_or_default().to_string())),
        Err(_) => Ok(None),
    }
}

pub async fn create_vnet(client: &AzureClient, location: &str) -> Result<String> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}",
        resource_group, VPC_NAME
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    let body = json!({
        "location": location,
        "tags": { "created-by": "byocvpn" },
        "properties": {
            "addressSpace": {
                "addressPrefixes": [
                    build_vnet_cidr_for_location(location),
                    build_vnet_ipv6_cidr_for_location(location)
                ]
            }
        }
    });

    info!("[Azure] Creating VNet '{}' in {}...", VPC_NAME, location);
    let async_op = put_with_provider_retry(client, &url, &body, &format!("VNet '{}'", VPC_NAME))
        .await
        .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        })?;

    if let Some(op_url) = async_op {
        client.wait_for_async_operation(&op_url).await?;
    }

    let vnet = client.get(&url).await?;
    info!("[Azure] VNet '{}' created in {}.", VPC_NAME, location);
    Ok(vnet["id"].as_str().unwrap_or_default().to_string())
}

pub async fn get_subnet(client: &AzureClient, location: &str) -> Result<Option<String>> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/subnets/{}",
        resource_group, VPC_NAME, SUBNET_NAME
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    match client.get(&url).await {
        Ok(existing) => {
            let has_ipv6 = existing["properties"]["addressPrefixes"]
                .as_array()
                .map(|prefixes| {
                    prefixes.iter().any(|prefix| {
                        prefix
                            .as_str()
                            .is_some_and(|prefix_str| prefix_str.contains(':'))
                    })
                })
                .unwrap_or(false);
            if has_ipv6 {
                Ok(Some(
                    existing["id"].as_str().unwrap_or_default().to_string(),
                ))
            } else {
                info!(
                    "[Azure] Subnet '{}' exists but lacks IPv6 — will recreate as dual-stack.",
                    SUBNET_NAME
                );
                Ok(None)
            }
        }
        Err(_) => Ok(None),
    }
}

pub async fn create_subnet(client: &AzureClient, location: &str, nsg_id: &str) -> Result<String> {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/subnets/{}",
        resource_group, VPC_NAME, SUBNET_NAME
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    let cidr = build_subnet_cidr_for_location(location);
    let cidr_ipv6 = build_subnet_ipv6_cidr_for_location(location);

    let body = json!({
        "properties": {
            "addressPrefixes": [cidr, cidr_ipv6],
            "networkSecurityGroup": { "id": nsg_id }
        }
    });

    info!(
        "[Azure] Creating subnet '{}' ({}, {}) in {}...",
        SUBNET_NAME, cidr, cidr_ipv6, location
    );
    let async_op =
        put_with_provider_retry(client, &url, &body, &format!("subnet '{}'", SUBNET_NAME))
            .await
            .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                reason: error.to_string(),
            })?;

    if let Some(op_url) = async_op {
        client.wait_for_async_operation(&op_url).await?;
    }

    let subnet = client.get(&url).await?;
    info!(
        "[Azure] Subnet '{}' ({}) created in {}.",
        SUBNET_NAME, cidr, location
    );
    Ok(subnet["id"].as_str().unwrap_or_default().to_string())
}

// ── VM network resources (public IPs, NIC) ───────────────────────────────────

#[derive(Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
}

impl IpVersion {
    fn get_resource_suffix(self) -> &'static str {
        match self {
            Self::V4 => "pip4",
            Self::V6 => "pip6",
        }
    }

    fn get_address_version(self) -> &'static str {
        match self {
            Self::V4 => "IPv4",
            Self::V6 => "IPv6",
        }
    }
}

fn build_public_ip_resource_name(vm_name: &str, version: IpVersion) -> String {
    format!("{}-{}", vm_name, version.get_resource_suffix())
}

fn build_public_ip_url(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> String {
    let resource_group = build_resource_group_name(location);
    let name = build_public_ip_resource_name(vm_name, version);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, name
    ));
    client.build_arm_url(&path, API_VERSION_NETWORK)
}

pub async fn get_public_ip_id(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> Result<Option<String>> {
    let url = build_public_ip_url(client, location, vm_name, version);
    match client.get(&url).await {
        Ok(body) => Ok(Some(body["id"].as_str().unwrap_or_default().to_string())),
        Err(_) => Ok(None),
    }
}

pub async fn create_public_ip_address(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> Result<String> {
    let url = build_public_ip_url(client, location, vm_name, version);

    let body = json!({
        "location": location,
        "sku": { "name": "Standard" },
        "tags": { "created-by": "byocvpn" },
        "properties": {
            "publicIPAllocationMethod": "Static",
            "publicIPAddressVersion": version.get_address_version()
        }
    });

    let async_op_url = client.put(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: format!(
                "Failed to create {} public IP: {}",
                version.get_address_version(),
                error
            ),
        }
    })?;

    if let Some(op_url) = async_op_url {
        client.wait_for_async_operation(&op_url).await?;
    }

    let public_ip = client.get(&url).await?;
    Ok(public_ip["id"].as_str().unwrap_or_default().to_string())
}

pub async fn get_public_ip_address(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> Result<String> {
    let url = build_public_ip_url(client, location, vm_name, version);
    let public_ip = client.get(&url).await?;
    Ok(public_ip["properties"]["ipAddress"]
        .as_str()
        .unwrap_or_default()
        .to_string())
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
    let resource_group = build_resource_group_name(location);
    let nic_name = format!("{}-nic", vm_name);

    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}",
        resource_group, nic_name
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    let body = json!({
        "location": location,
        "tags": { "created-by": "byocvpn" },
        "properties": {
            "networkSecurityGroup": { "id": nsg_id },
            "ipConfigurations": [
                {
                    "name": "ipconfig1",
                    "properties": {
                        "privateIPAllocationMethod": "Dynamic",
                        "privateIPAddressVersion": "IPv4",
                        "subnet": { "id": subnet_id },
                        "publicIPAddress": { "id": public_ipv4_id },
                        "primary": true
                    }
                },
                {
                    "name": "ipconfig2",
                    "properties": {
                        "privateIPAllocationMethod": "Dynamic",
                        "privateIPAddressVersion": "IPv6",
                        "subnet": { "id": subnet_id },
                        "publicIPAddress": { "id": public_ipv6_id }
                    }
                }
            ]
        }
    });

    let async_op_url = client.put(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: format!("Failed to create NIC: {}", error),
        }
    })?;

    if let Some(op_url) = async_op_url {
        client.wait_for_async_operation(&op_url).await?;
    }

    let nic = client.get(&url).await?;
    Ok(nic["id"].as_str().unwrap_or_default().to_string())
}

async fn delete_network_resource(
    client: &AzureClient,
    location: &str,
    resource_type: &str,
    resource_name: &str,
) {
    let resource_group = build_resource_group_name(location);
    let path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/{}/{}",
        resource_group, resource_type, resource_name
    ));
    let url = client.build_arm_url(&path, API_VERSION_NETWORK);

    match client.delete(&url).await {
        Ok(Some(op_url)) => {
            if let Err(error) = client.wait_for_async_operation(&op_url).await {
                error!(
                    "[Azure] Failed to wait for {} deletion: {}",
                    resource_name, error
                );
            }
        }
        Ok(None) => {}
        Err(error) => error!("[Azure] Failed to delete {}: {}", resource_name, error),
    }
}

pub async fn delete_nic(client: &AzureClient, location: &str, vm_name: &str) {
    delete_network_resource(
        client,
        location,
        "networkInterfaces",
        &format!("{}-nic", vm_name),
    )
    .await;
}

async fn delete_public_ip(client: &AzureClient, location: &str, vm_name: &str, version: IpVersion) {
    let name = build_public_ip_resource_name(vm_name, version);
    delete_network_resource(client, location, "publicIPAddresses", &name).await;
}

pub async fn cleanup_vm_resources(client: &AzureClient, location: &str, vm_name: &str) {
    delete_nic(client, location, vm_name).await;
    tokio::join!(
        delete_public_ip(client, location, vm_name, IpVersion::V4),
        delete_public_ip(client, location, vm_name, IpVersion::V6),
    );
}
