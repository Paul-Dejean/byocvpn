use std::collections::HashMap;

use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
    managed_resources::{
        MANAGED_BY_TAG_KEY, MANAGED_BY_TAG_VALUE, RESOURCE_NAME_PREFIX, SPAWN_ID_TAG_KEY,
    },
    server_startup::{PackageManager, generate_server_startup_script},
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    client::GcpClient,
    models::{
        AccessConfig, AttachedDisk, CreateInstanceRequest, DiskInitializeParams, InstanceMetadata,
        InstanceResponse, InstanceTags, Ipv6AccessConfig, MetadataItem, NetworkInterface,
        Operation, ZoneInstanceListResponse,
    },
    network::{RegionNetwork, firewall::FIREWALL_TARGET_TAG},
    operations::wait_for_compute_operation,
    state::GcpInstanceStatus,
};

const MACHINE_TYPE: &str = "e2-micro";
const DISK_TYPE: &str = "pd-standard";
const DISK_SIZE_GB: &str = "10";
const PRIMARY_ZONE_SUFFIX: &str = "a";
const INSTANCE_NAME_RANDOM_LENGTH: usize = 12;
const MAX_LISTED_INSTANCES: u32 = 500;

pub async fn spawn_instance(
    client: &GcpClient,
    region_network: &RegionNetwork,
    image_self_link: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let region = params.region;
    let zone = build_primary_zone(region);
    let instance_name = build_instance_name();
    let build_spawn_error = |reason: String| ComputeProvisioningError::InstanceSpawnFailed {
        region_name: region.to_string(),
        reason,
    };

    let body = build_create_instance_request(
        client,
        &zone,
        &instance_name,
        &region_network.subnet_self_link,
        image_self_link,
        params,
    )?;
    let operation: Operation = client
        .post(
            &client.build_compute_url(&format!("zones/{zone}/instances")),
            &body,
        )
        .await
        .map_err(|error| build_spawn_error(error.to_string()))?;
    wait_for_compute_operation(client, &operation)
        .await
        .map_err(|error| build_spawn_error(error.to_string()))?;

    let instance: InstanceResponse = client
        .get(&client.build_compute_url(&format!("zones/{zone}/instances/{instance_name}")))
        .await?;

    Ok(InstanceInfo {
        id: format!("{zone}/{instance_name}"),
        name: Some(instance_name),
        region: region.to_string(),
        state: InstanceState::Running,
        error_reason: None,
        public_ip_v4: extract_public_ip_v4(&instance),
        public_ip_v6: extract_public_ip_v6(&instance),
        provider: CloudProviderName::Gcp,
        instance_type: MACHINE_TYPE.to_string(),
        launched_at: Some(Utc::now()),
        spawn_id: Some(params.spawn_id.to_string()),
    })
}

pub async fn terminate_instance(client: &GcpClient, instance_id: &str) -> Result<()> {
    let (zone, instance_name) = parse_instance_id(instance_id)?;
    let url = client.build_compute_url(&format!("zones/{zone}/instances/{instance_name}"));
    client.delete(&url).await.map_err(|error| {
        ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_id.to_string(),
            reason: error.to_string(),
        }
        .into()
    })
}

pub async fn list_instances(client: &GcpClient, region: &str) -> Result<Vec<InstanceInfo>> {
    list_managed_instances(client, Some(region)).await
}

pub async fn list_all_instances(client: &GcpClient) -> Result<Vec<InstanceInfo>> {
    list_managed_instances(client, None).await
}

fn build_create_instance_request(
    client: &GcpClient,
    zone: &str,
    instance_name: &str,
    subnet_self_link: &str,
    image_self_link: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<CreateInstanceRequest> {
    let startup_script = generate_server_startup_script(
        PackageManager::Apt,
        params.server_private_key,
        params.client_public_key,
    )?;
    let labels = HashMap::from([
        (
            MANAGED_BY_TAG_KEY.to_string(),
            MANAGED_BY_TAG_VALUE.to_string(),
        ),
        (SPAWN_ID_TAG_KEY.to_string(), params.spawn_id.to_string()),
    ]);

    Ok(CreateInstanceRequest {
        name: instance_name.to_string(),
        machine_type: client
            .build_resource_link(&format!("zones/{zone}/machineTypes/{MACHINE_TYPE}")),
        disks: vec![AttachedDisk {
            boot: true,
            auto_delete: true,
            initialize_params: DiskInitializeParams {
                source_image: image_self_link.to_string(),
                disk_size_gb: DISK_SIZE_GB.to_string(),
                disk_type: client
                    .build_resource_link(&format!("zones/{zone}/diskTypes/{DISK_TYPE}")),
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
            items: vec![FIREWALL_TARGET_TAG.to_string()],
        },
    })
}

async fn list_managed_instances(
    client: &GcpClient,
    region_filter: Option<&str>,
) -> Result<Vec<InstanceInfo>> {
    let url = client.build_compute_url(&format!(
        "aggregated/instances?filter=labels.{MANAGED_BY_TAG_KEY}%3D{MANAGED_BY_TAG_VALUE}&maxResults={MAX_LISTED_INSTANCES}"
    ));
    let response: ZoneInstanceListResponse = client.get(&url).await?;

    Ok(response
        .items
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(zone_key, zone_instances)| {
            let zone = zone_key.trim_start_matches("zones/").to_string();
            zone_instances
                .instances
                .unwrap_or_default()
                .into_iter()
                .map(move |instance| (zone.clone(), instance))
        })
        .filter_map(|(zone, instance)| {
            let region = extract_region_from_zone(&zone);
            if region_filter.is_some_and(|wanted_region| wanted_region != region) {
                return None;
            }
            build_instance_info(&instance, &zone, &region)
        })
        .filter(|instance| instance.state.is_listed())
        .collect())
}

fn build_instance_info(
    instance: &InstanceResponse,
    zone: &str,
    region: &str,
) -> Option<InstanceInfo> {
    let name = instance.name.clone()?;
    let state = GcpInstanceStatus::from_state_name(instance.status.as_deref().unwrap_or_default());
    let instance_type = instance
        .machine_type
        .as_deref()
        .and_then(|machine_type_link| machine_type_link.rsplit('/').next())
        .unwrap_or(MACHINE_TYPE)
        .to_string();
    let launched_at = instance
        .creation_timestamp
        .as_deref()
        .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|datetime| datetime.with_timezone(&Utc));
    let spawn_id = instance
        .labels
        .as_ref()
        .and_then(|labels| labels.get(SPAWN_ID_TAG_KEY))
        .cloned();

    Some(InstanceInfo {
        id: format!("{zone}/{name}"),
        name: Some(name),
        region: region.to_string(),
        state: state.into(),
        error_reason: None,
        public_ip_v4: extract_public_ip_v4(instance),
        public_ip_v6: extract_public_ip_v6(instance),
        provider: CloudProviderName::Gcp,
        instance_type,
        launched_at,
        spawn_id,
    })
}

fn extract_public_ip_v4(instance: &InstanceResponse) -> String {
    instance
        .network_interfaces
        .as_ref()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface.access_configs.as_ref())
        .and_then(|access_configs| access_configs.first())
        .and_then(|access_config| access_config.nat_ip.clone())
        .unwrap_or_default()
}

fn extract_public_ip_v6(instance: &InstanceResponse) -> String {
    instance
        .network_interfaces
        .as_ref()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface.ipv6_access_configs.as_ref())
        .and_then(|access_configs| access_configs.first())
        .and_then(|access_config| access_config.external_ipv6.clone())
        .unwrap_or_default()
}

fn build_primary_zone(region: &str) -> String {
    format!("{region}-{PRIMARY_ZONE_SUFFIX}")
}

fn extract_region_from_zone(zone: &str) -> String {
    zone.rsplit_once('-')
        .map_or(zone, |(region, _zone_suffix)| region)
        .to_string()
}

fn build_instance_name() -> String {
    format!(
        "{RESOURCE_NAME_PREFIX}-{}",
        &Uuid::new_v4().simple().to_string()[..INSTANCE_NAME_RANDOM_LENGTH]
    )
}

fn parse_instance_id(instance_id: &str) -> Result<(&str, &str)> {
    instance_id
        .split_once('/')
        .ok_or_else(|| ComputeProvisioningError::MissingInstanceIdentifier.into())
}
