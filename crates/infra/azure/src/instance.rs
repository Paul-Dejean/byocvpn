use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    error::{ComputeProvisioningError, Error, Result},
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use log::*;
use serde_json::json;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

use crate::{
    client::AzureClient,
    network::{
        IpVersion, build_resource_group_name, cleanup_vm_resources, create_nic,
        create_public_ip_address, ensure_providers_registered, ensure_region_networking,
        get_public_ip_address, get_public_ip_id,
    },
    startup_script::generate_server_startup_script,
    state::AzureProvisioningState,
};

const VM_SIZES: &[&str] = &[
    "Standard_B1s",
    "Standard_B1ms",
    "Standard_B2s",
    "Standard_D2s_v3",
    "Standard_D2s_v4",
    "Standard_D2s_v5",
    "Standard_F2s_v2",
];
const IMAGE_PUBLISHER: &str = "Canonical";
const IMAGE_OFFER: &str = "0001-com-ubuntu-server-jammy";
const IMAGE_SKU: &str = "22_04-lts-gen2";
const API_VERSION_COMPUTE: &str = "2024-07-01";

fn build_spawn_error(location: &str, context: &str, error: impl std::fmt::Display) -> Error {
    let reason = format!("{}: {}", context, error);
    error!("[Azure] spawn error in {}: {}", location, reason);
    ComputeProvisioningError::InstanceSpawnFailed {
        region_name: location.to_string(),
        reason,
    }
    .into()
}

pub async fn spawn_instance(
    client: &AzureClient,
    location: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    ensure_providers_registered(client)
        .await
        .map_err(|error| build_spawn_error(location, "Provider registration", error))?;

    let network_ids = ensure_region_networking(client, location)
        .await
        .map_err(|error| build_spawn_error(location, "Region networking", error))?;

    let vm_name = format!(
        "byocvpn-{}",
        Uuid::new_v4().to_string().replace('-', "")[..12].to_lowercase()
    );

    let public_ipv4_id = match get_public_ip_id(client, location, &vm_name, IpVersion::V4).await? {
        Some(id) => id,
        None => create_public_ip_address(client, location, &vm_name, IpVersion::V4)
            .await
            .map_err(|error| build_spawn_error(location, "Public IP", error))?,
    };

    let public_ipv6_id = match get_public_ip_id(client, location, &vm_name, IpVersion::V6).await? {
        Some(id) => id,
        None => match create_public_ip_address(client, location, &vm_name, IpVersion::V6).await {
            Ok(id) => id,
            Err(error) => {
                cleanup_vm_resources(client, location, &vm_name).await;
                return Err(build_spawn_error(location, "Public IPv6 IP", error));
            }
        },
    };

    let nic_id = match create_nic(
        client,
        location,
        &vm_name,
        &network_ids.subnet_id,
        &public_ipv4_id,
        &public_ipv6_id,
        &network_ids.nsg_id,
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            cleanup_vm_resources(client, location, &vm_name).await;
            return Err(build_spawn_error(location, "NIC", error));
        }
    };

    let custom_data =
        generate_server_startup_script(params.server_private_key, params.client_public_key)
            .map_err(|error| build_spawn_error(location, "Startup script", error))?;

    let resource_group = build_resource_group_name(location);
    let vm_path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}",
        resource_group, vm_name
    ));
    let vm_url = client.build_arm_url(&vm_path, API_VERSION_COMPUTE);

    let admin_password = format!(
        "Byocvpn!{}",
        Uuid::new_v4().to_string().replace('-', "")[..16].to_uppercase()
    );

    let (async_op_url, used_vm_size) = {
        let mut last_error = String::new();
        let mut result = None;
        for &vm_size in VM_SIZES {
            let vm_body = json!({
                "location": location,
                "tags": { "created-by": "byocvpn" },
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
                    info!("[Azure] VM '{}' creating with size {}...", vm_name, vm_size);
                    result = Some((op_url, vm_size));
                    break;
                }
                Err(error) => {
                    last_error = error.to_string();
                    if last_error.contains("SkuNotAvailable") {
                        warn!(
                            "[Azure] {} not available in {}, trying next size...",
                            vm_size, location
                        );
                    } else {
                        cleanup_vm_resources(client, location, &vm_name).await;
                        return Err(build_spawn_error(location, "VM creation", &last_error));
                    }
                }
            }
        }
        match result {
            Some((op_url, vm_size)) => (op_url, vm_size),
            None => {
                cleanup_vm_resources(client, location, &vm_name).await;
                return Err(build_spawn_error(
                    location,
                    "VM creation",
                    format!("No available VM size in {}: {}", location, last_error),
                ));
            }
        }
    };

    if let Some(op_url) = async_op_url
        && let Err(error) = wait_for_vm_creation(client, &op_url, location).await
    {
        cleanup_vm_resources(client, location, &vm_name).await;
        return Err(error);
    }

    let public_ip_v4 = get_public_ip_address(client, location, &vm_name, IpVersion::V4)
        .await
        .unwrap_or_default();
    let public_ip_v6 = get_public_ip_address(client, location, &vm_name, IpVersion::V6)
        .await
        .unwrap_or_default();

    let instance_id = format!("{}/{}", resource_group, vm_name);
    info!("[Azure] VM '{}' created in {}.", vm_name, location);

    Ok(InstanceInfo {
        id: instance_id,
        name: Some(vm_name),
        region: location.to_string(),
        state: InstanceState::Running,
        public_ip_v4,
        public_ip_v6,
        provider: CloudProviderName::Azure,
        instance_type: used_vm_size.to_string(),
        launched_at: Some(Utc::now()),
    })
}

pub async fn terminate_instance(client: &AzureClient, instance_id: &str) -> Result<()> {
    let (resource_group, vm_name) = parse_instance_id(instance_id)?;

    let location = resource_group
        .strip_prefix("byocvpn-")
        .unwrap_or(resource_group);

    let vm_path = client.build_subscription_path(&format!(
        "/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}",
        resource_group, vm_name
    ));
    let vm_url = client.build_arm_url(&vm_path, API_VERSION_COMPUTE);

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

    info!("[Azure] VM '{}' deleted.", vm_name);

    cleanup_vm_resources(client, location, vm_name).await;

    Ok(())
}

pub async fn list_instances(client: &AzureClient, location: &str) -> Result<Vec<InstanceInfo>> {
    let all = list_all_instances(client).await?;
    Ok(all
        .into_iter()
        .filter(|instance| instance.region == location)
        .collect())
}

pub async fn list_all_instances(client: &AzureClient) -> Result<Vec<InstanceInfo>> {
    let path = client.build_subscription_path("/providers/Microsoft.Compute/virtualMachines");
    let url = client.build_arm_url(&path, API_VERSION_COMPUTE);

    let response = client.get(&url).await?;

    let vms: Vec<serde_json::Value> = response["value"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|virtual_machine| {
            virtual_machine["tags"]["created-by"]
                .as_str()
                .map(|tag_value| tag_value == "byocvpn")
                .unwrap_or(false)
        })
        .collect();

    let futures: Vec<_> = vms
        .iter()
        .map(|virtual_machine| async move { resolve_vm_info(client, virtual_machine).await })
        .collect();

    let results = join_all(futures).await;

    let mut instances = Vec::new();
    for result in results {
        match result {
            Ok(Some(instance)) => instances.push(instance),
            Ok(None) => {}
            Err(error) => error!("[Azure] Failed to resolve VM info: {}", error),
        }
    }

    Ok(instances)
}

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

    let resource_group = crate::client::extract_resource_group_from_id(vm_id)
        .unwrap_or("unknown")
        .to_string();

    let state: InstanceState = AzureProvisioningState::from(
        vm["properties"]["provisioningState"]
            .as_str()
            .unwrap_or("Unknown"),
    )
    .into();

    let instance_type = vm["properties"]["hardwareProfile"]["vmSize"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let launched_at = vm["properties"]["timeCreated"]
        .as_str()
        .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|datetime| datetime.with_timezone(&Utc));

    let public_ip_v4 = get_public_ip_address(client, &location, &vm_name, IpVersion::V4)
        .await
        .unwrap_or_default();
    let public_ip_v6 = get_public_ip_address(client, &location, &vm_name, IpVersion::V6)
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
        provider: CloudProviderName::Azure,
        instance_type,
        launched_at,
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
                debug!(
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
