/// Azure VM lifecycle: spawn, terminate, list.
///
/// Each VM is named `byocvpn-{uuid[..12]}` and tagged `byocvpn: true` so
/// that it can be found via the subscription-level VM list API.
///
/// Per-VM network resources follow a deterministic naming convention so
/// that termination can clean them up without storing extra state:
///
/// - NIC:        `{vm_name}-nic`
/// - Public IP:  `{vm_name}-pip4`
use byocvpn_core::{
    cloud_provider::{InstanceInfo, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
};
use futures::future::join_all;
use serde_json::json;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

use crate::{
    client::AzureClient,
    cloud_init::generate_wireguard_startup_script,
    network::{
        create_nic, delete_nic, delete_public_ip, delete_public_ipv6, ensure_nsg,
        ensure_providers_registered, ensure_public_ip, ensure_public_ipv6, ensure_resource_group,
        ensure_vnet_and_subnet, get_public_ipv4, get_public_ipv6, resource_group_for_location,
    },
};

/// VM sizes tried in order — falls back to the next on SkuNotAvailable.
const VM_SIZES: &[&str] = &[
    "Standard_B1s",    // 1 vCPU, 1 GB  — cheapest, Gen 2
    "Standard_B1ms",   // 1 vCPU, 2 GB, Gen 2
    "Standard_B2s",    // 2 vCPU, 4 GB, Gen 2
    "Standard_D2s_v3", // 2 vCPU, 8 GB, Gen 2, widely available
    "Standard_D2s_v4", // 2 vCPU, 8 GB, Gen 2
    "Standard_D2s_v5", // 2 vCPU, 8 GB, Gen 2
    "Standard_F2s_v2", // 2 vCPU, 4 GB, Gen 2, compute-optimised
];
const IMAGE_PUBLISHER: &str = "Canonical";
const IMAGE_OFFER: &str = "0001-com-ubuntu-server-jammy";
const IMAGE_SKU: &str = "22_04-lts-gen2";
const API_VERSION_COMPUTE: &str = "2024-07-01";

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Launch a new byocvpn WireGuard VM in `location`.
///
/// Infrastructure is provisioned idempotently before the VM is created:
/// resource group → NSG → VNet + subnet → public IP → NIC → VM.
pub async fn spawn_instance(
    client: &AzureClient,
    location: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    // Register required ARM namespaces — safe no-op if already registered.
    ensure_providers_registered(client).await.map_err(|error| {
        let reason = format!("Provider registration: {}", error);
        eprintln!("[Azure] spawn error in {}: {}", location, reason);
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: location.to_string(),
            reason,
        }
    })?;

    // Ensure shared infrastructure exists.
    ensure_resource_group(client, location)
        .await
        .map_err(|error| {
            let reason = format!("Resource group: {}", error);
            eprintln!("[Azure] spawn error in {}: {}", location, reason);
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason,
            }
        })?;

    let subnet_id = ensure_vnet_and_subnet(client, location)
        .await
        .map_err(|error| {
            let reason = format!("VNet/subnet: {}", error);
            eprintln!("[Azure] spawn error in {}: {}", location, reason);
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason,
            }
        })?;

    let nsg_id = ensure_nsg(client, location).await.map_err(|error| {
        let reason = format!("NSG: {}", error);
        eprintln!("[Azure] spawn error in {}: {}", location, reason);
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: location.to_string(),
            reason,
        }
    })?;

    // Generate a short, unique VM name.
    let vm_name = format!(
        "byocvpn-{}",
        Uuid::new_v4().to_string().replace('-', "")[..12].to_lowercase()
    );

    // Create the public IPv4 IP.
    let pip_id = ensure_public_ip(client, location, &vm_name)
        .await
        .map_err(|error| {
            let reason = format!("Public IP: {}", error);
            eprintln!("[Azure] spawn error in {}: {}", location, reason);
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason,
            }
        })?;

    // Create the public IPv6 IP — roll back the IPv4 IP on failure.
    let pip6_id = match ensure_public_ipv6(client, location, &vm_name).await {
        Ok(id) => id,
        Err(error) => {
            let reason = format!("Public IPv6 IP: {}", error);
            eprintln!("[Azure] spawn error in {}: {}", location, reason);
            delete_public_ip(client, location, &vm_name).await;
            return Err(ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason,
            }
            .into());
        }
    };

    // Create the NIC — roll back both public IPs on failure.
    let nic_id = match create_nic(
        client, location, &vm_name, &subnet_id, &pip_id, &pip6_id, &nsg_id,
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            let reason = format!("NIC: {}", error);
            eprintln!("[Azure] spawn error in {}: {}", location, reason);
            delete_public_ip(client, location, &vm_name).await;
            delete_public_ipv6(client, location, &vm_name).await;
            return Err(ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason,
            }
            .into());
        }
    };

    // Render and base64-encode the WireGuard startup script.
    let custom_data =
        generate_wireguard_startup_script(params.server_private_key, params.client_public_key)
            .map_err(|error| {
                let reason = format!("Startup script: {}", error);
                eprintln!("[Azure] spawn error in {}: {}", location, reason);
                ComputeProvisioningError::InstanceSpawnFailed {
                    region_name: location.to_string(),
                    reason,
                }
            })?;

    // Build the VM body.
    let resource_group = resource_group_for_location(location);
    let vm_path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}",
        resource_group, vm_name
    ));
    let vm_url = client.arm_url(&vm_path, API_VERSION_COMPUTE);

    // Use a random admin password (not shared with the user — access is via WireGuard).
    let admin_password = format!(
        "Byocvpn!{}",
        Uuid::new_v4().to_string().replace('-', "")[..16].to_uppercase()
    );

    let async_op_url = {
        let mut last_error = String::new();
        let mut result = None;
        for &vm_size in VM_SIZES {
            let vm_body = json!({
                "location": location,
                "tags": { "byocvpn": "true" },
                "properties": {
                    "hardwareProfile": { "vmSize": vm_size },
                    "osProfile": {
                        "computerName": vm_name,
                        "adminUsername": "azureuser",
                        "adminPassword": admin_password,
                        "linuxConfiguration": {
                            "disablePasswordAuthentication": false,
                            "provisionVMAgent": true
                        },
                        "customData": custom_data
                    },
                    "storageProfile": {
                        "imageReference": {
                            "publisher": IMAGE_PUBLISHER,
                            "offer": IMAGE_OFFER,
                            "sku": IMAGE_SKU,
                            "version": "latest"
                        },
                        "osDisk": {
                            "createOption": "FromImage",
                            "deleteOption": "Delete",
                            "diskSizeGB": 30
                        }
                    },
                    "networkProfile": {
                        "networkInterfaces": [{
                            "id": nic_id,
                            "properties": { "deleteOption": "Delete" }
                        }]
                    }
                }
            });
            match client.put(&vm_url, &vm_body).await {
                Ok(op_url) => {
                    eprintln!("[Azure] VM '{}' creating with size {}...", vm_name, vm_size);
                    result = Some(op_url);
                    break;
                }
                Err(error) => {
                    last_error = error.to_string();
                    if last_error.contains("SkuNotAvailable") {
                        eprintln!(
                            "[Azure] {} not available in {}, trying next size...",
                            vm_size, location
                        );
                    } else {
                        let reason = last_error.clone();
                        eprintln!(
                            "[Azure] spawn error in {}: VM PUT failed: {}",
                            location, reason
                        );
                        delete_nic(client, location, &vm_name).await;
                        delete_public_ip(client, location, &vm_name).await;
                        delete_public_ipv6(client, location, &vm_name).await;
                        return Err(ComputeProvisioningError::InstanceSpawnFailed {
                            region_name: location.to_string(),
                            reason,
                        }
                        .into());
                    }
                }
            }
        }
        match result {
            Some(op_url) => op_url,
            None => {
                eprintln!(
                    "[Azure] All VM sizes exhausted in {}: {}",
                    location, last_error
                );
                delete_nic(client, location, &vm_name).await;
                delete_public_ip(client, location, &vm_name).await;
                delete_public_ipv6(client, location, &vm_name).await;
                return Err(ComputeProvisioningError::InstanceSpawnFailed {
                    region_name: location.to_string(),
                    reason: format!("No available VM size in {}: {}", location, last_error),
                }
                .into());
            }
        }
    };

    // Wait for the VM creation to complete (typically 1–3 minutes).
    if let Some(op_url) = async_op_url {
        if let Err(error) = wait_for_vm_creation(client, &op_url, location).await {
            delete_nic(client, location, &vm_name).await;
            delete_public_ip(client, location, &vm_name).await;
            delete_public_ipv6(client, location, &vm_name).await;
            return Err(error);
        }
    }

    // Fetch the allocated public IPv4 and IPv6 addresses.
    let public_ip_v4 = get_public_ipv4(client, location, &vm_name)
        .await
        .unwrap_or_default();
    let public_ip_v6 = get_public_ipv6(client, location, &vm_name)
        .await
        .unwrap_or_default();

    let instance_id = format!("{}/{}", resource_group, vm_name);
    println!("[Azure] VM '{}' created in {}.", vm_name, location);

    Ok(InstanceInfo {
        id: instance_id,
        name: Some(vm_name),
        region: location.to_string(),
        state: "Running".to_string(),
        public_ip_v4,
        public_ip_v6,
        provider: "azure".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Terminate
// ---------------------------------------------------------------------------

/// Terminate a VM and clean up its NIC + public IP.
///
/// `instance_id` has the format `{resource_group}/{vm_name}`.
pub async fn terminate_instance(client: &AzureClient, instance_id: &str) -> Result<()> {
    let (resource_group, vm_name) = parse_instance_id(instance_id)?;

    // Derive the location from the resource group name (`byocvpn-{location}`).
    let location = resource_group
        .strip_prefix("byocvpn-")
        .unwrap_or(resource_group);

    let vm_path = client.subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}",
        resource_group, vm_name
    ));
    let vm_url = client.arm_url(&vm_path, API_VERSION_COMPUTE);

    // Delete the VM.
    let async_op_url = client.delete(&vm_url).await.map_err(|error| {
        ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_id.to_string(),
            reason: error.to_string(),
        }
    })?;

    if let Some(op_url) = async_op_url {
        client
            .wait_for_async_operation(&op_url)
            .await
            .map_err(
                |error| ComputeProvisioningError::InstanceTerminationFailed {
                    instance_identifier: instance_id.to_string(),
                    reason: error.to_string(),
                },
            )?;
    }

    println!("[Azure] VM '{}' deleted.", vm_name);

    // Clean up NIC and public IPs (best-effort; VM is already gone).
    delete_nic(client, location, vm_name).await;
    delete_public_ip(client, location, vm_name).await;
    delete_public_ipv6(client, location, vm_name).await;

    Ok(())
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// List all byocvpn instances in a specific Azure location.
pub async fn list_instances(client: &AzureClient, location: &str) -> Result<Vec<InstanceInfo>> {
    let all = list_all_instances(client).await?;
    Ok(all
        .into_iter()
        .filter(|instance| instance.region == location)
        .collect())
}

/// List all byocvpn instances across the entire subscription.
///
/// Uses the subscription-level VM list API and filters by the `byocvpn` tag.
/// Public IPs are resolved concurrently.
pub async fn list_all_instances(client: &AzureClient) -> Result<Vec<InstanceInfo>> {
    let path = client.subscription_path("/providers/Microsoft.Compute/virtualMachines");
    let url = client.arm_url(&path, API_VERSION_COMPUTE);

    let response = client.get(&url).await?;

    let vms: Vec<serde_json::Value> = response["value"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|vm| {
            // Keep only VMs tagged with byocvpn: true.
            vm["tags"]["byocvpn"]
                .as_str()
                .map(|v| v == "true")
                .unwrap_or(false)
        })
        .collect();

    // Resolve public IPs concurrently.
    let futures: Vec<_> = vms
        .iter()
        .map(|vm| async move { resolve_vm_info(client, vm).await })
        .collect();

    let results = join_all(futures).await;

    let mut instances = Vec::new();
    for result in results {
        match result {
            Ok(Some(instance)) => instances.push(instance),
            Ok(None) => {}
            Err(error) => eprintln!("[Azure] Failed to resolve VM info: {}", error),
        }
    }

    Ok(instances)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn resolve_vm_info(
    client: &AzureClient,
    vm: &serde_json::Value,
) -> Result<Option<InstanceInfo>> {
    let vm_name = match vm["name"].as_str() {
        Some(name) => name.to_string(),
        None => return Ok(None),
    };
    let location = match vm["location"].as_str() {
        Some(loc) => loc.to_string(),
        None => return Ok(None),
    };
    let vm_id = match vm["id"].as_str() {
        Some(id) => id,
        None => return Ok(None),
    };

    let resource_group = crate::client::resource_group_from_id(vm_id)
        .unwrap_or("unknown")
        .to_string();

    let state = vm["properties"]["provisioningState"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    // Try to resolve the public IPs; tolerate failures (VM may still be provisioning).
    let public_ip_v4 = get_public_ipv4(client, &location, &vm_name)
        .await
        .unwrap_or_default();
    let public_ip_v6 = get_public_ipv6(client, &location, &vm_name)
        .await
        .unwrap_or_default();

    let instance_id = format!("{}/{}", resource_group, vm_name);

    Ok(Some(InstanceInfo {
        id: instance_id,
        name: Some(vm_name),
        region: location,
        state,
        public_ip_v4,
        public_ip_v6,
        provider: "azure".to_string(),
    }))
}

fn parse_instance_id(instance_id: &str) -> Result<(&str, &str)> {
    let mut parts = instance_id.splitn(2, '/');
    let resource_group = parts
        .next()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?;
    let vm_name = parts
        .next()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?;
    Ok((resource_group, vm_name))
}

/// Poll the VM creation async-operation with appropriate delays.
async fn wait_for_vm_creation(
    client: &AzureClient,
    operation_url: &str,
    location: &str,
) -> Result<()> {
    for attempt in 1..=60u32 {
        let body = client.get(operation_url).await?;
        match body["status"].as_str() {
            Some("Succeeded") => return Ok(()),
            Some("Failed") | Some("Canceled") => {
                let message = body["error"]["message"].as_str().unwrap_or("unknown error");
                return Err(ComputeProvisioningError::InstanceSpawnFailed {
                    region_name: location.to_string(),
                    reason: message.to_string(),
                }
                .into());
            }
            _ => {
                eprintln!(
                    "[Azure] Waiting for VM creation (attempt {}/60)...",
                    attempt
                );
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
    Err(ComputeProvisioningError::InstanceSpawnFailed {
        region_name: location.to_string(),
        reason: "Timed out waiting for VM creation".to_string(),
    }
    .into())
}
