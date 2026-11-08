mod vm_network;

use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
    managed_resources::{
        MANAGED_BY_TAG_KEY, MANAGED_BY_TAG_VALUE, RESOURCE_NAME_PREFIX, SPAWN_ID_TAG_KEY,
    },
    server_startup::{PackageManager, generate_server_startup_script},
};
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    arm_resource::{
        API_VERSION_COMPUTE, ArmResource, IpVersion, build_resource_group_name,
        extract_resource_group_from_id,
    },
    client::AzureClient,
    error::{ArmErrorCode, extract_arm_error_code},
    image::build_image_reference,
    instance::vm_network::{
        PublicIpAddresses, cleanup_vm_resources, create_nic, ensure_public_ip,
        get_public_ip_address, list_public_ip_addresses,
    },
    models::{
        HardwareProfile, LinuxConfiguration, NetworkInterfaceReference, NetworkProfile, OsDisk,
        OsProfile, StorageProfile, VmListResponse, VmProperties, VmRequest, VmResponse,
        build_managed_tags,
    },
    network::RegionNetwork,
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
pub const OS_DISK_SIZE_GB: u32 = 30;
const ADMIN_USERNAME: &str = "azureuser";
const VM_NAME_RANDOM_LENGTH: usize = 12;
const ADMIN_PASSWORD_RANDOM_LENGTH: usize = 16;

pub async fn spawn_instance(
    client: &AzureClient,
    region_network: &RegionNetwork,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let location = params.region;
    info!("[Azure] Spawning instance in '{location}'...");
    let vm_name = build_vm_name();
    let resource_group = build_resource_group_name(location);

    match provision_virtual_machine(client, region_network, &resource_group, &vm_name, params).await
    {
        Ok(instance) => {
            info!("[Azure] VM '{vm_name}' created in {location}.");
            Ok(instance)
        }
        Err(error) => {
            error!("[Azure] Spawn in {location} failed, cleaning up VM '{vm_name}': {error}");
            cleanup_vm_resources(client, &resource_group, &vm_name).await;
            Err(error.with_context(format!("spawning instance in {location}")))
        }
    }
}

pub async fn terminate_instance(client: &AzureClient, instance_id: &str) -> Result<()> {
    info!("[Azure] Terminating instance '{instance_id}'...");
    let (resource_group, vm_name) = parse_instance_id(instance_id)?;
    let vm_url = client.build_resource_url(&ArmResource::virtual_machine(resource_group, vm_name));

    let build_termination_error =
        |reason: String| ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_id.to_string(),
            reason,
        };
    let operation_url = client
        .delete(&vm_url)
        .await
        .map_err(|error| build_termination_error(error.to_string()))?;
    if let Some(operation_url) = operation_url {
        client
            .wait_for_async_operation(&operation_url)
            .await
            .map_err(|error| build_termination_error(error.to_string()))?;
    }

    info!("[Azure] VM '{vm_name}' deleted.");
    cleanup_vm_resources(client, resource_group, vm_name).await;
    Ok(())
}

pub async fn list_instances(client: &AzureClient, location: &str) -> Result<Vec<InstanceInfo>> {
    let all_instances = list_all_instances(client).await?;
    Ok(all_instances
        .into_iter()
        .filter(|instance| instance.region == location)
        .collect())
}

pub async fn list_all_instances(client: &AzureClient) -> Result<Vec<InstanceInfo>> {
    debug!("[Azure] Listing all virtual machines across subscription...");
    let path = client.build_subscription_path("/providers/Microsoft.Compute/virtualMachines");
    let url = AzureClient::build_arm_url(&path, API_VERSION_COMPUTE);
    let response: VmListResponse = client.get(&url).await?;

    let managed_vms: Vec<ManagedVm> = response
        .value
        .unwrap_or_default()
        .into_iter()
        .filter(is_managed_vm)
        .filter_map(ManagedVm::from_response)
        .collect();
    let addresses_by_resource_group =
        list_addresses_by_resource_group(client, &managed_vms).await?;

    let instances: Vec<InstanceInfo> = managed_vms
        .iter()
        .filter_map(|managed_vm| {
            let addresses = addresses_by_resource_group.get(managed_vm.resource_group.as_str())?;
            Some(managed_vm.build_instance_info(addresses))
        })
        .filter(|instance| instance.state.is_listed())
        .collect();
    debug!(
        "[Azure] Found {} byocvpn instance(s) in subscription.",
        instances.len()
    );
    Ok(instances)
}

async fn provision_virtual_machine(
    client: &AzureClient,
    region_network: &RegionNetwork,
    resource_group: &str,
    vm_name: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let location = params.region;
    let nic_id = create_vm_network(client, region_network, location, vm_name).await?;
    let custom_data = build_startup_script(params)?;
    let launch_spec = VmLaunchSpec {
        location,
        vm_name,
        nic_id: &nic_id,
        custom_data: &custom_data,
        admin_password: &build_admin_password(),
        spawn_id: params.spawn_id,
    };

    let vm_url = client.build_resource_url(&ArmResource::virtual_machine(resource_group, vm_name));
    let (operation_url, vm_size) =
        create_vm_with_size_fallback(client, &vm_url, &launch_spec).await?;
    if let Some(operation_url) = operation_url {
        client
            .wait_for_async_operation(&operation_url)
            .await
            .map_err(|error| error.with_context("waiting for virtual machine creation"))?;
    }

    let public_ip_v4 = read_public_ip_or_warn(client, location, vm_name, IpVersion::V4).await;
    let public_ip_v6 = read_public_ip_or_warn(client, location, vm_name, IpVersion::V6).await;
    let launched_at = read_creation_time_or_now(client, &vm_url).await;

    Ok(InstanceInfo {
        id: format!("{resource_group}/{vm_name}"),
        name: Some(vm_name.to_string()),
        region: location.to_string(),
        state: InstanceState::Running,
        error_reason: None,
        public_ip_v4,
        public_ip_v6,
        provider: CloudProviderName::Azure,
        instance_type: vm_size.to_string(),
        launched_at: Some(launched_at),
        spawn_id: Some(params.spawn_id.to_string()),
    })
}

async fn create_vm_network(
    client: &AzureClient,
    region_network: &RegionNetwork,
    location: &str,
    vm_name: &str,
) -> Result<String> {
    let public_ipv4_id = ensure_public_ip(client, location, vm_name, IpVersion::V4).await?;
    let public_ipv6_id = ensure_public_ip(client, location, vm_name, IpVersion::V6).await?;
    create_nic(
        client,
        location,
        vm_name,
        &region_network.subnet_id,
        &public_ipv4_id,
        &public_ipv6_id,
        &region_network.network_security_group_id,
    )
    .await
}

fn build_startup_script(params: &SpawnInstanceParams<'_>) -> Result<String> {
    let script = generate_server_startup_script(
        PackageManager::Apt,
        params.server_private_key,
        params.client_public_key,
    )?;
    Ok(BASE64.encode(script.as_bytes()))
}

struct VmLaunchSpec<'a> {
    location: &'a str,
    vm_name: &'a str,
    nic_id: &'a str,
    custom_data: &'a str,
    admin_password: &'a str,
    spawn_id: &'a str,
}

async fn create_vm_with_size_fallback(
    client: &AzureClient,
    vm_url: &str,
    launch_spec: &VmLaunchSpec<'_>,
) -> Result<(Option<String>, &'static str)> {
    let location = launch_spec.location;
    let mut last_unavailable_size_error = None;
    for &vm_size in VM_SIZES {
        let body = build_vm_request(launch_spec, vm_size);
        match client.put(vm_url, &body).await {
            Ok(operation_url) => {
                info!(
                    "[Azure] VM '{}' creating with size {vm_size}...",
                    launch_spec.vm_name
                );
                return Ok((operation_url, vm_size));
            }
            Err(error) if extract_arm_error_code(&error) == Some(ArmErrorCode::SkuNotAvailable) => {
                warn!("[Azure] {vm_size} not available in {location}, trying next size...");
                last_unavailable_size_error = Some(error);
            }
            Err(error) => return Err(error.with_context("creating virtual machine")),
        }
    }

    Err(last_unavailable_size_error.map_or_else(
        || {
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: location.to_string(),
                reason: "no VM sizes configured".to_string(),
            }
            .into()
        },
        |error| error.with_context(format!("no available VM size in {location}")),
    ))
}

fn build_vm_request(launch_spec: &VmLaunchSpec<'_>, vm_size: &str) -> VmRequest {
    let mut tags = build_managed_tags();
    tags.insert(
        SPAWN_ID_TAG_KEY.to_string(),
        launch_spec.spawn_id.to_string(),
    );

    VmRequest {
        location: launch_spec.location.to_string(),
        tags,
        properties: VmProperties {
            hardware_profile: HardwareProfile {
                vm_size: vm_size.to_string(),
            },
            os_profile: OsProfile {
                computer_name: launch_spec.vm_name.to_string(),
                admin_username: ADMIN_USERNAME.to_string(),
                admin_password: launch_spec.admin_password.to_string(),
                linux_configuration: LinuxConfiguration {
                    disable_password_authentication: false,
                    provision_vm_agent: true,
                },
                custom_data: launch_spec.custom_data.to_string(),
            },
            storage_profile: StorageProfile {
                image_reference: build_image_reference(),
                os_disk: OsDisk {
                    create_option: "FromImage".to_string(),
                    delete_option: "Delete".to_string(),
                    disk_size_gb: OS_DISK_SIZE_GB,
                },
            },
            network_profile: NetworkProfile {
                network_interfaces: vec![NetworkInterfaceReference {
                    id: launch_spec.nic_id.to_string(),
                }],
            },
        },
    }
}

async fn read_public_ip_or_warn(
    client: &AzureClient,
    location: &str,
    vm_name: &str,
    version: IpVersion,
) -> String {
    match get_public_ip_address(client, location, vm_name, version).await {
        Ok(address) => address,
        Err(error) => {
            warn!(
                "[Azure] Could not read {} address of VM '{vm_name}': {error}",
                version.get_address_version()
            );
            String::new()
        }
    }
}

async fn read_creation_time_or_now(client: &AzureClient, vm_url: &str) -> DateTime<Utc> {
    match client.get::<VmResponse>(vm_url).await {
        Ok(virtual_machine) => parse_creation_time(&virtual_machine).unwrap_or_else(Utc::now),
        Err(error) => {
            warn!("[Azure] Could not read VM creation time, using current time: {error}");
            Utc::now()
        }
    }
}

fn parse_creation_time(virtual_machine: &VmResponse) -> Option<DateTime<Utc>> {
    virtual_machine
        .properties
        .as_ref()
        .and_then(|properties| properties.time_created.as_deref())
        .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|datetime| datetime.with_timezone(&Utc))
}

fn build_vm_name() -> String {
    format!(
        "{RESOURCE_NAME_PREFIX}-{}",
        build_random_suffix(VM_NAME_RANDOM_LENGTH).to_lowercase()
    )
}

fn build_admin_password() -> String {
    format!(
        "Byocvpn!{}",
        build_random_suffix(ADMIN_PASSWORD_RANDOM_LENGTH).to_uppercase()
    )
}

fn build_random_suffix(length: usize) -> String {
    Uuid::new_v4().simple().to_string()[..length].to_string()
}

async fn list_addresses_by_resource_group<'a>(
    client: &AzureClient,
    managed_vms: &'a [ManagedVm],
) -> Result<HashMap<&'a str, PublicIpAddresses>> {
    let mut resource_groups: Vec<&str> = managed_vms
        .iter()
        .map(|managed_vm| managed_vm.resource_group.as_str())
        .collect();
    resource_groups.sort_unstable();
    resource_groups.dedup();

    let addresses = try_join_all(
        resource_groups
            .iter()
            .map(|resource_group| list_public_ip_addresses(client, resource_group)),
    )
    .await?;
    Ok(resource_groups.into_iter().zip(addresses).collect())
}

fn is_managed_vm(virtual_machine: &VmResponse) -> bool {
    virtual_machine
        .tags
        .as_ref()
        .and_then(|tags| tags.get(MANAGED_BY_TAG_KEY))
        .is_some_and(|tag_value| tag_value == MANAGED_BY_TAG_VALUE)
}

struct ManagedVm {
    name: String,
    location: String,
    resource_group: String,
    state: InstanceState,
    instance_type: String,
    launched_at: Option<DateTime<Utc>>,
    spawn_id: Option<String>,
}

impl ManagedVm {
    fn from_response(virtual_machine: VmResponse) -> Option<Self> {
        let launched_at = parse_creation_time(&virtual_machine);
        let name = virtual_machine.name?;
        let location = virtual_machine.location?;
        let resource_group = extract_resource_group_from_id(virtual_machine.id.as_deref()?)
            .unwrap_or("unknown")
            .to_string();
        let properties = virtual_machine.properties;

        let state = AzureProvisioningState::from_state_name(
            properties
                .as_ref()
                .and_then(|properties| properties.provisioning_state.as_deref())
                .unwrap_or_default(),
        )
        .into();
        let instance_type = properties
            .as_ref()
            .and_then(|properties| properties.hardware_profile.as_ref())
            .and_then(|hardware_profile| hardware_profile.vm_size.clone())
            .unwrap_or_default();
        let spawn_id = virtual_machine
            .tags
            .as_ref()
            .and_then(|tags| tags.get(SPAWN_ID_TAG_KEY))
            .cloned();

        Some(Self {
            name,
            location,
            resource_group,
            state,
            instance_type,
            launched_at,
            spawn_id,
        })
    }

    fn build_instance_info(&self, addresses: &PublicIpAddresses) -> InstanceInfo {
        InstanceInfo {
            id: format!("{}/{}", self.resource_group, self.name),
            name: Some(self.name.clone()),
            region: self.location.clone(),
            state: self.state.clone(),
            error_reason: None,
            public_ip_v4: addresses.get_address(&self.resource_group, &self.name, IpVersion::V4),
            public_ip_v6: addresses.get_address(&self.resource_group, &self.name, IpVersion::V6),
            provider: CloudProviderName::Azure,
            instance_type: self.instance_type.clone(),
            launched_at: self.launched_at,
            spawn_id: self.spawn_id.clone(),
        }
    }
}

fn parse_instance_id(instance_id: &str) -> Result<(&str, &str)> {
    instance_id
        .split_once('/')
        .ok_or_else(|| ComputeProvisioningError::MissingInstanceIdentifier.into())
}
