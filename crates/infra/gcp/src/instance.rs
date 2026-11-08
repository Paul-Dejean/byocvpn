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
use google_cloud_compute_v1::model::{
    AccessConfig, AttachedDisk, AttachedDiskInitializeParams, Instance, Metadata, NetworkInterface,
    Tags,
    access_config::{NetworkTier, Type as AccessConfigType},
    metadata::Items as MetadataItem,
    network_interface::StackType,
};
use uuid::Uuid;

use crate::{
    client::GcpClient,
    error::convert_sdk_error,
    network::{RegionNetwork, firewall::FIREWALL_TARGET_TAG},
    operations::await_compute_operation,
    state::map_instance_state,
};

const MACHINE_TYPE: &str = "e2-micro";
const DISK_TYPE: &str = "pd-standard";
const DISK_SIZE_GB: i64 = 10;
const PRIMARY_ZONE_SUFFIX: &str = "a";
const INSTANCE_NAME_RANDOM_LENGTH: usize = 12;
const MAX_LISTED_INSTANCES: u32 = 500;
const STARTUP_SCRIPT_METADATA_KEY: &str = "startup-script";
const EXTERNAL_IPV4_ACCESS_NAME: &str = "External NAT";
const EXTERNAL_IPV6_ACCESS_NAME: &str = "External IPv6";

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

    let instance = build_instance_request(
        client,
        &zone,
        &instance_name,
        &region_network.subnet_self_link,
        image_self_link,
        params,
    )?;
    let insert = client
        .instances
        .insert()
        .set_project(&client.project_id)
        .set_zone(&zone)
        .set_body(instance)
        .poller();
    await_compute_operation("creating instance", insert)
        .await
        .map_err(|error| build_spawn_error(error.to_string()))?;

    let instance = client
        .instances
        .get()
        .set_project(&client.project_id)
        .set_zone(&zone)
        .set_instance(&instance_name)
        .send()
        .await
        .map_err(|error| convert_sdk_error("getting instance", error))?;

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
    let delete = client
        .instances
        .delete()
        .set_project(&client.project_id)
        .set_zone(zone)
        .set_instance(instance_name)
        .poller();
    match await_compute_operation("deleting instance", delete).await {
        Ok(()) => Ok(()),
        Err(error) if error.is_not_found() => Ok(()),
        Err(error) => Err(ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_id.to_string(),
            reason: error.to_string(),
        }
        .into()),
    }
}

pub async fn list_instances(client: &GcpClient, region: &str) -> Result<Vec<InstanceInfo>> {
    list_managed_instances(client, Some(region)).await
}

pub async fn list_all_instances(client: &GcpClient) -> Result<Vec<InstanceInfo>> {
    list_managed_instances(client, None).await
}

fn build_instance_request(
    client: &GcpClient,
    zone: &str,
    instance_name: &str,
    subnet_self_link: &str,
    image_self_link: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<Instance> {
    let startup_script = generate_server_startup_script(
        PackageManager::Apt,
        params.server_private_key,
        params.client_public_key,
    )?;
    let labels = HashMap::from([
        (MANAGED_BY_TAG_KEY, MANAGED_BY_TAG_VALUE),
        (SPAWN_ID_TAG_KEY, params.spawn_id),
    ]);

    let boot_disk = AttachedDisk::new()
        .set_boot(true)
        .set_auto_delete(true)
        .set_initialize_params(
            AttachedDiskInitializeParams::new()
                .set_source_image(image_self_link)
                .set_disk_size_gb(DISK_SIZE_GB)
                .set_disk_type(
                    client.build_resource_link(&format!("zones/{zone}/diskTypes/{DISK_TYPE}")),
                ),
        );
    let network_interface = NetworkInterface::new()
        .set_subnetwork(subnet_self_link)
        .set_stack_type(StackType::Ipv4Ipv6)
        .set_access_configs([AccessConfig::new()
            .set_type(AccessConfigType::OneToOneNat)
            .set_name(EXTERNAL_IPV4_ACCESS_NAME)
            .set_network_tier(NetworkTier::Premium)])
        .set_ipv_6_access_configs([AccessConfig::new()
            .set_type(AccessConfigType::DirectIpv6)
            .set_name(EXTERNAL_IPV6_ACCESS_NAME)
            .set_network_tier(NetworkTier::Premium)]);

    Ok(Instance::new()
        .set_name(instance_name)
        .set_machine_type(
            client.build_resource_link(&format!("zones/{zone}/machineTypes/{MACHINE_TYPE}")),
        )
        .set_disks([boot_disk])
        .set_network_interfaces([network_interface])
        .set_metadata(
            Metadata::new().set_items([MetadataItem::new()
                .set_key(STARTUP_SCRIPT_METADATA_KEY)
                .set_value(startup_script)]),
        )
        .set_labels(labels)
        .set_tags(Tags::new().set_items([FIREWALL_TARGET_TAG])))
}

async fn list_managed_instances(
    client: &GcpClient,
    region_filter: Option<&str>,
) -> Result<Vec<InstanceInfo>> {
    let response = client
        .instances
        .aggregated_list()
        .set_project(&client.project_id)
        .set_filter(format!(
            "labels.{MANAGED_BY_TAG_KEY}={MANAGED_BY_TAG_VALUE}"
        ))
        .set_max_results(MAX_LISTED_INSTANCES)
        .send()
        .await
        .map_err(|error| convert_sdk_error("listing instances", error))?;

    Ok(response
        .items
        .into_iter()
        .flat_map(|(zone_key, zone_instances)| {
            let zone = zone_key.trim_start_matches("zones/").to_string();
            zone_instances
                .instances
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

fn build_instance_info(instance: &Instance, zone: &str, region: &str) -> Option<InstanceInfo> {
    let name = instance.name.clone()?;
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

    Some(InstanceInfo {
        id: format!("{zone}/{name}"),
        name: Some(name),
        region: region.to_string(),
        state: map_instance_state(instance.status.as_ref()),
        error_reason: None,
        public_ip_v4: extract_public_ip_v4(instance),
        public_ip_v6: extract_public_ip_v6(instance),
        provider: CloudProviderName::Gcp,
        instance_type,
        launched_at,
        spawn_id: instance.labels.get(SPAWN_ID_TAG_KEY).cloned(),
    })
}

fn extract_public_ip_v4(instance: &Instance) -> String {
    instance
        .network_interfaces
        .first()
        .and_then(|interface| interface.access_configs.first())
        .and_then(|access_config| access_config.nat_ip.clone())
        .unwrap_or_default()
}

fn extract_public_ip_v6(instance: &Instance) -> String {
    instance
        .network_interfaces
        .first()
        .and_then(|interface| interface.ipv_6_access_configs.first())
        .and_then(|access_config| access_config.external_ipv_6.clone())
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
