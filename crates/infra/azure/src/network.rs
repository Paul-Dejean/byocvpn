/// Azure network infrastructure provisioning.
///
/// All shared resources (resource group, NSG, VNet, subnet) are created
/// once per region under the naming convention `byocvpn-{location}`.
/// They are idempotent: if a resource already exists the operation is a no-op.
use byocvpn_core::error::{NetworkProvisioningError, Result};
use serde_json::json;
use tokio::time::{Duration, sleep};

use crate::client::AzureClient;

// ARM API versions
const API_VERSION_RESOURCE_GROUPS: &str = "2021-04-01";
const API_VERSION_NETWORK: &str = "2024-05-01";
const API_VERSION_LOCATIONS: &str = "2022-12-01";

// Resource naming
const VPC_NAME: &str = "byocvpn-vnet";
const SUBNET_NAME: &str = "byocvpn-subnet";
const NSG_NAME: &str = "byocvpn-nsg";

use serde::Serialize;

/// Resource group name for a given Azure location.
pub fn resource_group_for_location(location: &str) -> String {
    format!("byocvpn-{}", location)
}

// ---------------------------------------------------------------------------
// Retry helper
// ---------------------------------------------------------------------------

/// Wrapper around [`AzureClient::put`] that retries indefinitely on
/// `MissingSubscriptionRegistration` (Azure 409).
///
/// Provider registration can take up to 10+ minutes to propagate in some
/// regions.  This helper waits up to 20 minutes (120 × 10 s) before giving up.
async fn put_with_provider_retry<B: Serialize>(
    client: &AzureClient,
    url: &str,
    body: &B,
    resource_label: &str,
) -> Result<Option<String>> {
    let max_attempts = 120u32; // 120 × 10 s = 20 min
    let mut last_error = String::new();
    for attempt in 1..=max_attempts {
        match client.put(url, body).await {
            Ok(op_url) => return Ok(op_url),
            Err(error) => {
                last_error = error.to_string();
                if last_error.contains("MissingSubscriptionRegistration") {
                    eprintln!("[Azure] {} — 409 response: {}", resource_label, last_error);
                    eprintln!(
                        "[Azure] {} — re-registering and retrying in 15 s (attempt {}/{}, {}s elapsed)...",
                        resource_label,
                        attempt,
                        max_attempts,
                        attempt * 15
                    );
                    // Re-POST the registration on every attempt — idempotent
                    // and may unstick stalled regional propagation.
                    let register_path =
                        client.subscription_path("/providers/Microsoft.Network/register");
                    let register_url = client.arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
                    match client.post(&register_url, &serde_json::Value::Null).await {
                        Ok(body) => eprintln!("[Azure] Re-registration POST succeeded: {}", body),
                        Err(reg_error) => {
                            eprintln!("[Azure] Re-registration POST failed: {}", reg_error)
                        }
                    }
                    sleep(Duration::from_secs(15)).await;
                } else {
                    return Err(NetworkProvisioningError::NetworkQueryFailed {
                        reason: format!("{} PUT failed: {}", resource_label, last_error),
                    }
                    .into());
                }
            }
        }
    }
    Err(NetworkProvisioningError::NetworkQueryFailed {
        reason: format!(
            "{} PUT timed out after {} retries ({}s): {}",
            resource_label,
            max_attempts,
            max_attempts * 10,
            last_error
        ),
    }
    .into())
}

// ---------------------------------------------------------------------------
// Provider Registration
// ---------------------------------------------------------------------------

/// Ensure the required ARM resource-provider namespaces are registered on the
/// subscription.  New or free-tier subscriptions often ship with providers
/// unregistered; calling `register` on an already-registered provider is a
/// safe no-op.
pub async fn ensure_providers_registered(client: &AzureClient) -> Result<()> {
    const NAMESPACES: &[&str] = &[
        "Microsoft.Network",
        "Microsoft.Compute",
        "Microsoft.Resources",
    ];

    for namespace in NAMESPACES {
        // Check current state first — skip the POST if already registered.
        let status_path = client.subscription_path(&format!("/providers/{}", namespace));
        let status_url = client.arm_url(&status_path, API_VERSION_RESOURCE_GROUPS);
        let state = client
            .get(&status_url)
            .await
            .ok()
            .and_then(|body| body["registrationState"].as_str().map(|s| s.to_string()));

        if state.as_deref() == Some("Registered") {
            continue;
        }

        // Trigger registration.
        let register_path = client.subscription_path(&format!("/providers/{}/register", namespace));
        let register_url = client.arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
        eprintln!("[Azure] Registering provider '{}'...", namespace);
        client
            .post(&register_url, &serde_json::Value::Null)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to register provider {}: {}", namespace, error),
            })?;

        // Poll until Registered (typically 30–90 s).
        for attempt in 1..=60u32 {
            sleep(Duration::from_secs(5)).await;
            let body = client.get(&status_url).await.map_err(|error| {
                NetworkProvisioningError::NetworkQueryFailed {
                    reason: format!("Failed to poll provider {}: {}", namespace, error),
                }
            })?;
            let registration_state = body["registrationState"].as_str().unwrap_or("Unknown");
            eprintln!(
                "[Azure] Provider '{}' registration state: {} (attempt {}/60)",
                namespace, registration_state, attempt
            );
            if registration_state == "Registered" {
                break;
            }
            if attempt == 60 {
                return Err(NetworkProvisioningError::NetworkQueryFailed {
                    reason: format!("Timed out waiting for provider '{}' to register", namespace),
                }
                .into());
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Resource Group
// ---------------------------------------------------------------------------

/// Ensure the `byocvpn-{location}` resource group exists.
pub async fn ensure_resource_group(client: &AzureClient, location: &str) -> Result<()> {
    let resource_group = resource_group_for_location(location);
    let path = client.subscription_path(&format!("/resourceGroups/{}", resource_group));
    let url = client.arm_url(&path, API_VERSION_RESOURCE_GROUPS);

    // GET first — returns 200 if it already exists.
    if client.get(&url).await.is_ok() {
        return Ok(());
    }

    let body = json!({
        "location": location,
        "tags": { "byocvpn": "true" }
    });

    eprintln!("[Azure] Creating resource group '{}'...", resource_group);
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
        eprintln!(
            "[Azure] Waiting for resource group '{}' to be ready...",
            resource_group
        );
        client.wait_for_async_operation(&op_url).await?;
    }

    println!("[Azure] Resource group '{}' created.", resource_group);
    Ok(())
}

// ---------------------------------------------------------------------------
// Network Security Group
// ---------------------------------------------------------------------------

/// Ensure the `byocvpn-nsg` NSG exists with a UDP 51820 inbound rule.
///
/// Returns the NSG resource ID.
pub async fn ensure_nsg(client: &AzureClient, location: &str) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups/{}",
        resource_group, NSG_NAME
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    if let Ok(existing) = client.get(&url).await {
        return Ok(existing["id"].as_str().unwrap_or_default().to_string());
    }

    let body = json!({
        "location": location,
        "tags": { "byocvpn": "true" },
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

    eprintln!("[Azure] Creating NSG '{}' in {}...", NSG_NAME, location);
    let async_op_url = put_with_provider_retry(client, &url, &body, &format!("NSG '{}'", NSG_NAME))
        .await
        .map_err(
            |error| NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            },
        )?;

    if let Some(op_url) = async_op_url {
        eprintln!(
            "[Azure] Waiting for NSG '{}' to be provisioned...",
            NSG_NAME
        );
        client.wait_for_async_operation(&op_url).await?;
    }

    // Fetch to get the resource ID.
    let nsg = client.get(&url).await?;
    println!("[Azure] NSG '{}' created in {}.", NSG_NAME, location);
    Ok(nsg["id"].as_str().unwrap_or_default().to_string())
}

// ---------------------------------------------------------------------------
// Virtual Network + Subnet
// ---------------------------------------------------------------------------

/// Ensure the `byocvpn-vnet` VNet and `byocvpn-subnet` subnet exist.
///
/// Returns the subnet resource ID.
pub async fn ensure_vnet_and_subnet(client: &AzureClient, location: &str) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let nsg_id = ensure_nsg(client, location).await?;

    let vnet_path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}",
        resource_group, VPC_NAME
    ));
    let vnet_url = client.arm_url(&vnet_path, API_VERSION_NETWORK);

    let subnet_path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/subnets/{}",
        resource_group, VPC_NAME, SUBNET_NAME
    ));
    let subnet_url = client.arm_url(&subnet_path, API_VERSION_NETWORK);

    let cidr = subnet_cidr_for_location(location);
    let cidr_ipv6 = subnet_ipv6_cidr_for_location(location);

    // If the subnet already exists with dual-stack (IPv4 + IPv6), return early.
    // If it only has IPv4, fall through so we can upgrade it.
    if let Ok(existing_subnet) = client.get(&subnet_url).await {
        let has_ipv6 = existing_subnet["properties"]["addressPrefixes"]
            .as_array()
            .map(|prefixes| {
                prefixes
                    .iter()
                    .any(|p| p.as_str().map_or(false, |s| s.contains(':')))
            })
            .unwrap_or(false);
        if has_ipv6 {
            return Ok(existing_subnet["id"]
                .as_str()
                .unwrap_or_default()
                .to_string());
        }
        eprintln!(
            "[Azure] Subnet '{}' exists but lacks IPv6 — upgrading to dual-stack...",
            SUBNET_NAME
        );
    }

    // Create or update the VNet (idempotent PUT).
    let vnet_body = json!({
        "location": location,
        "tags": { "byocvpn": "true" },
        "properties": {
            "addressSpace": {
                "addressPrefixes": [
                    vnet_cidr_for_location(location),
                    vnet_ipv6_cidr_for_location(location)
                ]
            }
        }
    });

    let vnet_async_op = put_with_provider_retry(
        client,
        &vnet_url,
        &vnet_body,
        &format!("VNet '{}'", VPC_NAME),
    )
    .await
    .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
        reason: error.to_string(),
    })?;

    if let Some(op_url) = vnet_async_op {
        client.wait_for_async_operation(&op_url).await?;
    }

    println!(
        "[Azure] VNet '{}' created/verified in {}.",
        VPC_NAME, location
    );

    // Create the subnet.
    let subnet_body = json!({
        "properties": {
            "addressPrefixes": [cidr, cidr_ipv6],
            "networkSecurityGroup": { "id": nsg_id }
        }
    });

    let subnet_async_op = put_with_provider_retry(
        client,
        &subnet_url,
        &subnet_body,
        &format!("subnet '{}'", SUBNET_NAME),
    )
    .await
    .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
        reason: error.to_string(),
    })?;

    if let Some(op_url) = subnet_async_op {
        client.wait_for_async_operation(&op_url).await?;
    }

    let subnet = client.get(&subnet_url).await?;
    println!(
        "[Azure] Subnet '{}' ({}) created in {}.",
        SUBNET_NAME, cidr, location
    );
    Ok(subnet["id"].as_str().unwrap_or_default().to_string())
}

// ---------------------------------------------------------------------------
// Regions
// ---------------------------------------------------------------------------

/// Return all Azure locations available to the subscription, mapped to a
/// `(location_name, continent)` pair.
///
/// Only `regionCategory: "Recommended"` locations are returned so that the
/// user does not see experimental or geography-restricted previews.
pub async fn list_regions(client: &AzureClient) -> Result<Vec<(String, String)>> {
    let path = client.subscription_path("/locations");
    let url = client.arm_url(&path, API_VERSION_LOCATIONS);

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
            // Restrict to recommended regions (skip deprecated / canary / etc.)
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

// ---------------------------------------------------------------------------
// Public IP address per VM
// ---------------------------------------------------------------------------

/// Ensure an IPv4 Standard public IP exists for a given VM name.
///
/// Returns the public IP resource ID.
pub async fn ensure_public_ip(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip4", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    let body = json!({
        "location": location,
        "sku": { "name": "Standard" },
        "tags": { "byocvpn": "true" },
        "properties": {
            "publicIPAllocationMethod": "Static",
            "publicIPAddressVersion": "IPv4"
        }
    });

    let async_op_url = client.put(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: format!("Failed to create public IP: {}", error),
        }
    })?;

    if let Some(op_url) = async_op_url {
        client.wait_for_async_operation(&op_url).await?;
    }

    let pip = client.get(&url).await?;
    Ok(pip["id"].as_str().unwrap_or_default().to_string())
}

/// Return the allocated IPv4 address for a VM's public IP resource.
///
/// Returns an empty string if the IP has not yet been allocated.
pub async fn get_public_ipv4(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip4", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    let pip = client.get(&url).await?;
    Ok(pip["properties"]["ipAddress"]
        .as_str()
        .unwrap_or_default()
        .to_string())
}

/// Ensure a Standard static IPv6 public IP exists for a given VM.
///
/// Returns the public IP resource ID.
pub async fn ensure_public_ipv6(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip6", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    let body = json!({
        "location": location,
        "sku": { "name": "Standard" },
        "tags": { "byocvpn": "true" },
        "properties": {
            "publicIPAllocationMethod": "Static",
            "publicIPAddressVersion": "IPv6"
        }
    });

    let async_op_url = client.put(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: format!("Failed to create IPv6 public IP: {}", error),
        }
    })?;

    if let Some(op_url) = async_op_url {
        client.wait_for_async_operation(&op_url).await?;
    }

    let pip = client.get(&url).await?;
    Ok(pip["id"].as_str().unwrap_or_default().to_string())
}

/// Return the allocated IPv6 address for a VM's IPv6 public IP resource.
///
/// Returns an empty string if the IP has not yet been allocated.
pub async fn get_public_ipv6(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip6", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    let pip = client.get(&url).await?;
    Ok(pip["properties"]["ipAddress"]
        .as_str()
        .unwrap_or_default()
        .to_string())
}

// ---------------------------------------------------------------------------
// Network Interface Card
// ---------------------------------------------------------------------------

/// Create a NIC for a VM, attaching the given subnet, IPv4 public IP, and IPv6 public IP.
///
/// Returns the NIC resource ID.
pub async fn create_nic(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    subnet_id: &str,
    pip_id: &str,
    pip6_id: &str,
    nsg_id: &str,
) -> Result<String> {
    let resource_group = resource_group_for_location(location);
    let nic_name = format!("{}-nic", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}",
        resource_group, nic_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    let body = json!({
        "location": location,
        "tags": { "byocvpn": "true" },
        "properties": {
            "networkSecurityGroup": { "id": nsg_id },
            "ipConfigurations": [
                {
                    "name": "ipconfig1",
                    "properties": {
                        "privateIPAllocationMethod": "Dynamic",
                        "privateIPAddressVersion": "IPv4",
                        "subnet": { "id": subnet_id },
                        "publicIPAddress": { "id": pip_id },
                        "primary": true
                    }
                },
                {
                    "name": "ipconfig2",
                    "properties": {
                        "privateIPAllocationMethod": "Dynamic",
                        "privateIPAddressVersion": "IPv6",
                        "subnet": { "id": subnet_id },
                        "publicIPAddress": { "id": pip6_id }
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

/// Delete a NIC by VM name (best-effort; errors are logged, not propagated).
pub async fn delete_nic(client: &AzureClient, location: &str, vm_name: &str) {
    let resource_group = resource_group_for_location(location);
    let nic_name = format!("{}-nic", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}",
        resource_group, nic_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    match client.delete(&url).await {
        Ok(Some(op_url)) => {
            if let Err(error) = client.wait_for_async_operation(&op_url).await {
                eprintln!("[Azure] Failed to wait for NIC deletion: {}", error);
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!("[Azure] Failed to delete NIC {}: {}", nic_name, error),
    }
}

/// Delete an IPv4 public IP by VM name (best-effort).
pub async fn delete_public_ip(client: &AzureClient, location: &str, vm_name: &str) {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip4", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    match client.delete(&url).await {
        Ok(Some(op_url)) => {
            if let Err(error) = client.wait_for_async_operation(&op_url).await {
                eprintln!(
                    "[Azure] Failed to wait for IPv4 public IP deletion: {}",
                    error
                );
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!("[Azure] Failed to delete public IP {}: {}", pip_name, error),
    }
}

/// Delete an IPv6 public IP by VM name (best-effort).
pub async fn delete_public_ipv6(client: &AzureClient, location: &str, vm_name: &str) {
    let resource_group = resource_group_for_location(location);
    let pip_name = format!("{}-pip6", vm_name);

    let path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
        resource_group, pip_name
    ));
    let url = client.arm_url(&path, API_VERSION_NETWORK);

    match client.delete(&url).await {
        Ok(Some(op_url)) => {
            if let Err(error) = client.wait_for_async_operation(&op_url).await {
                eprintln!(
                    "[Azure] Failed to wait for IPv6 public IP deletion: {}",
                    error
                );
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!(
            "[Azure] Failed to delete IPv6 public IP {}: {}",
            pip_name, error
        ),
    }
}

// ---------------------------------------------------------------------------
// CIDR helpers
// ---------------------------------------------------------------------------

/// Return the `/16` VNet IPv4 address space for a location.
pub fn vnet_cidr_for_location(location: &str) -> String {
    let index = location_index(location);
    format!("10.{}.0.0/16", index)
}

/// Return the `/24` subnet IPv4 CIDR for a location.
///
/// Unique per location so that VNets in different regions never overlap.
pub fn subnet_cidr_for_location(location: &str) -> String {
    let index = location_index(location);
    format!("10.{}.0.0/24", index)
}

/// Return the `/48` VNet IPv6 ULA prefix for a location.
///
/// Uses the `fd{index:02x}::/48` ULA range, which is unique per location
/// and allows the VM to masquerade IPv6 traffic via a Standard IPv6 public IP.
pub fn vnet_ipv6_cidr_for_location(location: &str) -> String {
    let index = location_index(location);
    format!("fd{:02x}::/48", index)
}

/// Return the `/64` subnet IPv6 prefix for a location.
pub fn subnet_ipv6_cidr_for_location(location: &str) -> String {
    let index = location_index(location);
    format!("fd{:02x}::/64", index)
}

/// Map common Azure location names to a stable octet in the 50–200 range.
fn location_index(location: &str) -> u8 {
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

    // Fallback: hash the location name into the 150–200 range.
    let hash: u8 = location
        .bytes()
        .enumerate()
        .fold(0u32, |acc, (i, b)| {
            acc.wrapping_add((b as u32).wrapping_mul((i as u32).wrapping_add(1)))
        })
        .wrapping_rem(50) as u8;
    150 + hash
}

// All public functions in this module are directly accessible; no re-export
// block needed.
