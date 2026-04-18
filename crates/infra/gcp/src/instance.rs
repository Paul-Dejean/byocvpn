use std::collections::HashMap;

use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
    retry::retry,
};
use chrono::{DateTime, Utc};
use tokio::time::Duration;
use uuid::Uuid;

use crate::{
    client::GcpClient,
    models::{
        AccessConfig, AttachedDisk, CreateInstanceRequest, DiskInitializeParams, InstanceMetadata,
        InstanceResponse, InstanceTags, Ipv6AccessConfig, MetadataItem, NetworkInterface,
        Operation, ZoneInstanceListResponse, ZoneOperationResponse,
    },
    network::build_primary_zone_for_region,
    startup_script::generate_server_startup_script,
    state::GcpInstanceStatus,
};
use log::*;

const MACHINE_TYPE: &str = "e2-micro";
const DISK_TYPE_SUFFIX: &str = "pd-standard";
const INSTANCE_LABEL_KEY: &str = "created-by";
const INSTANCE_LABEL_VALUE: &str = "byocvpn";
const INSTANCE_TAG: &str = "byocvpn";

pub async fn spawn_instance(
    client: &GcpClient,
    subnet_self_link: &str,
    image_self_link: &str,
    region: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let zone = build_primary_zone_for_region(region);
    let startup_script =
        generate_server_startup_script(params.server_private_key, params.client_public_key)?;

    let instance_name = format!(
        "byocvpn-{}",
        Uuid::new_v4().to_string().replace('-', "")[..12].to_lowercase()
    );

    let machine_type_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/zones/{}/machineTypes/{}",
        client.project_id, zone, MACHINE_TYPE
    );

    let disk_type_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/zones/{}/diskTypes/{}",
        client.project_id, zone, DISK_TYPE_SUFFIX
    );

    let mut labels = HashMap::new();
    labels.insert(INSTANCE_LABEL_KEY.to_string(), INSTANCE_LABEL_VALUE.to_string());

    let body = CreateInstanceRequest {
        name: instance_name.clone(),
        machine_type: machine_type_url,
        disks: vec![AttachedDisk {
            boot: true,
            auto_delete: true,
            initialize_params: DiskInitializeParams {
                source_image: image_self_link.to_string(),
                disk_size_gb: "10".to_string(),
                disk_type: disk_type_url,
            },
        }],
        network_interfaces: vec![NetworkInterface {
            subnetwork: subnet_self_link.to_string(),
            stack_type: "IPV4_IPV6".to_string(),
            access_configs: vec![AccessConfig {
                access_type: "ONE_TO_ONE_NAT".to_string(),
                name: "External NAT".to_string(),
                network_tier: "PREMIUM".to_string(),
            }],
            ipv6_access_configs: Some(vec![Ipv6AccessConfig {
                access_type: "DIRECT_IPV6".to_string(),
                name: "External IPv6".to_string(),
                network_tier: "PREMIUM".to_string(),
            }]),
        }],
        metadata: InstanceMetadata {
            items: vec![MetadataItem {
                key: "startup-script".to_string(),
                value: startup_script,
            }],
        },
        labels,
        tags: InstanceTags {
            items: vec![INSTANCE_TAG.to_string()],
        },
    };

    let url = format!(
        "{}/zones/{}/instances",
        client.build_compute_base_url(),
        zone
    );
    let operation: Operation = client.post(&url, &body).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    let operation_url = operation
        .self_link
        .as_deref()
        .ok_or_else(|| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: "Missing selfLink in operation response".to_string(),
        })?
        .to_string();
    wait_for_zone_operation(client, &operation_url, region).await?;

    let instance = get_instance(client, &zone, &instance_name).await?;
    let public_ip_v4 = extract_public_ip_v4(&instance);
    let public_ip_v6 = extract_public_ip_v6(&instance);

    let id = format!("{}/{}", zone, instance_name);
    Ok(InstanceInfo {
        id,
        name: Some(instance_name),
        region: region.to_string(),
        state: InstanceState::Running,
        public_ip_v4,
        public_ip_v6,
        provider: CloudProviderName::Gcp,
        instance_type: MACHINE_TYPE.to_string(),
        launched_at: Some(Utc::now()),
    })
}

pub async fn terminate_instance(client: &GcpClient, instance_id: &str) -> Result<()> {
    let (zone, instance_name) = parse_instance_id(instance_id)?;
    let url = format!(
        "{}/zones/{}/instances/{}",
        client.build_compute_base_url(),
        zone,
        instance_name
    );
    client.delete(&url).await.map_err(|error| {
        ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_id.to_string(),
            reason: error.to_string(),
        }
        .into()
    })
}

pub async fn list_instances(client: &GcpClient, region: &str) -> Result<Vec<InstanceInfo>> {
    let url = format!(
        "{}/aggregated/instances?filter=labels.{label_key}%3D{label_value}&maxResults=500",
        client.build_compute_base_url(),
        label_key = INSTANCE_LABEL_KEY,
        label_value = INSTANCE_LABEL_VALUE,
    );
    let response: ZoneInstanceListResponse = client.get(&url).await?;

    let mut instances = Vec::new();
    if let Some(items) = response.items {
        for (zone_key, zone_data) in items {
            if !zone_key.starts_with(&format!("zones/{}", region)) {
                continue;
            }
            let zone = zone_key.trim_start_matches("zones/").to_string();
            if let Some(instance_list) = zone_data.instances {
                for instance in instance_list {
                    if let Some(info) = parse_instance_info(&instance, &zone, region) {
                        instances.push(info);
                    }
                }
            }
        }
    }
    Ok(instances)
}

pub async fn list_all_instances(client: &GcpClient) -> Result<Vec<InstanceInfo>> {
    let url = format!(
        "{}/aggregated/instances?filter=labels.{label_key}%3D{label_value}&maxResults=500",
        client.build_compute_base_url(),
        label_key = INSTANCE_LABEL_KEY,
        label_value = INSTANCE_LABEL_VALUE,
    );
    let response: ZoneInstanceListResponse = client.get(&url).await?;

    let mut instances = Vec::new();
    if let Some(items) = response.items {
        for (zone_key, zone_data) in items {
            let zone = zone_key.trim_start_matches("zones/").to_string();
            let region = extract_region_from_zone(&zone);
            if let Some(instance_list) = zone_data.instances {
                for instance in instance_list {
                    if let Some(info) = parse_instance_info(&instance, &zone, &region) {
                        instances.push(info);
                    }
                }
            }
        }
    }
    Ok(instances)
}

fn parse_instance_id(instance_id: &str) -> Result<(&str, &str)> {
    let mut parts = instance_id.splitn(2, '/');
    let zone = parts
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingInstanceIdentifier)?;
    let name = parts
        .next()
        .ok_or_else(|| ComputeProvisioningError::MissingInstanceIdentifier)?;
    Ok((zone, name))
}

pub async fn get_instance(
    client: &GcpClient,
    zone: &str,
    instance_name: &str,
) -> Result<InstanceResponse> {
    let url = format!(
        "{}/zones/{}/instances/{}",
        client.build_compute_base_url(),
        zone,
        instance_name
    );
    client.get(&url).await
}

fn extract_public_ip_v4(instance: &InstanceResponse) -> String {
    instance
        .network_interfaces
        .as_ref()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface.access_configs.as_ref())
        .and_then(|configs| configs.first())
        .and_then(|config| config.nat_ip.as_deref())
        .unwrap_or_default()
        .to_string()
}

fn extract_public_ip_v6(instance: &InstanceResponse) -> String {
    instance
        .network_interfaces
        .as_ref()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface.ipv6_access_configs.as_ref())
        .and_then(|configs| configs.first())
        .and_then(|config| config.external_ipv6.as_deref())
        .unwrap_or_default()
        .to_string()
}

fn extract_region_from_zone(zone: &str) -> String {
    let mut parts: Vec<&str> = zone.split('-').collect();
    if parts.len() > 2 {
        parts.pop();
    }
    parts.join("-")
}

fn parse_instance_info(
    instance: &InstanceResponse,
    zone: &str,
    region: &str,
) -> Option<InstanceInfo> {
    let name = instance.name.clone()?;
    let status: InstanceState =
        GcpInstanceStatus::from(instance.status.as_deref().unwrap_or("UNKNOWN")).into();
    let public_ip_v4 = extract_public_ip_v4(instance);
    let public_ip_v6 = extract_public_ip_v6(instance);
    let id = format!("{}/{}", zone, name);

    let instance_type = instance
        .machine_type
        .as_deref()
        .and_then(|machine_type_url| machine_type_url.split('/').last())
        .unwrap_or(MACHINE_TYPE)
        .to_string();

    let launched_at = instance
        .creation_timestamp
        .as_deref()
        .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|datetime| datetime.with_timezone(&Utc));

    Some(InstanceInfo {
        id,
        name: Some(name),
        region: region.to_string(),
        state: status,
        public_ip_v4,
        public_ip_v6,
        provider: CloudProviderName::Gcp,
        instance_type,
        launched_at,
    })
}

async fn wait_for_zone_operation(
    client: &GcpClient,
    operation_url: &str,
    region: &str,
) -> Result<()> {
    retry(
        || async move {
            let operation: ZoneOperationResponse = client.get(operation_url).await?;
            match operation.status.as_deref() {
                Some("DONE") => {
                    if let Some(error) = operation.error {
                        let message = error
                            .errors
                            .as_ref()
                            .and_then(|errors| errors.first())
                            .and_then(|detail| detail.message.as_deref())
                            .unwrap_or("unknown error");
                        return Err(ComputeProvisioningError::InstanceSpawnFailed {
                            region_name: region.to_string(),
                            reason: message.to_string(),
                        }
                        .into());
                    }
                    Ok(())
                }
                _ => {
                    debug!("[GCP] Waiting for instance creation...");
                    Err(ComputeProvisioningError::InstanceSpawnFailed {
                        region_name: region.to_string(),
                        reason: "Timed out waiting for instance creation operation".to_string(),
                    }
                    .into())
                }
            }
        },
        60,
        Duration::from_secs(3),
    )
    .await
}
