use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, SpawnInstanceParams},
    managed_resources::{MANAGED_BY_TAG_KEY, MANAGED_BY_TAG_VALUE, SPAWN_ID_TAG_KEY},
    retry::{RetryPolicy, retry_if},
    server_startup::{PackageManager, generate_server_startup_script},
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use log::{debug, info, warn};
use tokio::time::Duration;

use crate::{
    client::OciClient,
    error::{OracleError, Result},
    models::{
        AvailabilityDomainResponse, CreateVnicDetails, InstanceResponse, LaunchInstanceRequest,
        ShapeConfig, SourceDetails, VnicAttachmentResponse, VnicResponse, build_managed_tags,
    },
    network::RegionNetwork,
    state::OciLifecycleState,
};

const INSTANCE_SHAPE: &str = "VM.Standard.A1.Flex";
pub const INSTANCE_OCPUS: f32 = 1.0;
pub const INSTANCE_MEMORY_GB: f32 = 6.0;
const INSTANCE_DISPLAY_NAME: &str = "byocvpn-server";
const INSTANCE_RUNNING_POLL_POLICY: RetryPolicy = RetryPolicy::fixed(36, Duration::from_secs(5));
const MAX_CONSECUTIVE_LOOKUP_FAILURES: u32 = 6;

pub async fn spawn_instance(
    client: &OciClient,
    region_network: &RegionNetwork,
    image_ocid: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let region = params.region;
    let body = build_launch_request(client, region_network, image_ocid, params).await?;

    debug!(
        "[OCI] Launching instance in {region} (compartment {}, subnet {})",
        client.get_compartment_ocid(),
        region_network.subnet_id
    );
    let launched: InstanceResponse = client
        .post(&client.build_core_url("/instances"), &body)
        .await?;

    let instance_ocid = launched.id;
    debug!("[OCI] Instance {instance_ocid} created, waiting for RUNNING state");
    wait_until_running(client, &instance_ocid).await?;

    let details = get_instance(client, &instance_ocid).await?;
    let mut instance = build_instance_info(&details, region);
    let (public_ip_v4, public_ip_v6) = get_public_ips(client, &instance_ocid).await;
    instance.public_ip_v4 = public_ip_v4;
    instance.public_ip_v6 = public_ip_v6;
    instance.instance_type = INSTANCE_SHAPE.to_string();
    instance.launched_at = Some(Utc::now());
    info!(
        "[OCI] Instance {} spawned in {region} (IPv4 {}, IPv6 {})",
        instance.id, instance.public_ip_v4, instance.public_ip_v6
    );
    Ok(instance)
}

pub async fn terminate_instance(client: &OciClient, instance_ocid: &str) -> Result<()> {
    debug!("[OCI] Terminating instance {instance_ocid}");
    let url = client.build_core_url(&format!(
        "/instances/{instance_ocid}?preserveBootVolume=false"
    ));
    client.delete(&url).await?;
    info!("[OCI] Instance {instance_ocid} terminated");
    Ok(())
}

pub async fn list_instances(client: &OciClient, region: &str) -> Result<Vec<InstanceInfo>> {
    let url = client.build_core_url(&format!(
        "/instances?compartmentId={}",
        client.get_compartment_ocid()
    ));
    let response: Vec<InstanceResponse> = client.get(&url).await?;

    let mut instances: Vec<InstanceInfo> = response
        .iter()
        .filter(|instance| is_managed_instance(instance))
        .map(|instance| build_instance_info(instance, region))
        .filter(|instance| instance.state.is_listed())
        .collect();

    let public_ips = join_all(
        instances
            .iter()
            .map(|instance| get_public_ips(client, &instance.id)),
    )
    .await;
    for (instance, (public_ip_v4, public_ip_v6)) in instances.iter_mut().zip(public_ips) {
        instance.public_ip_v4 = public_ip_v4;
        instance.public_ip_v6 = public_ip_v6;
    }
    Ok(instances)
}

async fn build_launch_request(
    client: &OciClient,
    region_network: &RegionNetwork,
    image_ocid: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<LaunchInstanceRequest> {
    let startup_script = generate_server_startup_script(
        PackageManager::Apt,
        params.server_private_key,
        params.client_public_key,
    )?;
    let mut freeform_tags = build_managed_tags();
    freeform_tags.insert(SPAWN_ID_TAG_KEY.to_string(), params.spawn_id.to_string());

    Ok(LaunchInstanceRequest {
        compartment_id: client.get_compartment_ocid().to_string(),
        display_name: INSTANCE_DISPLAY_NAME.to_string(),
        availability_domain: get_first_availability_domain(client, params.region).await?,
        shape: INSTANCE_SHAPE.to_string(),
        shape_config: ShapeConfig {
            ocpus: INSTANCE_OCPUS,
            memory_in_g_bs: INSTANCE_MEMORY_GB,
        },
        source_details: SourceDetails {
            source_type: "image".to_string(),
            image_id: image_ocid.to_string(),
        },
        create_vnic_details: CreateVnicDetails {
            subnet_id: region_network.subnet_id.clone(),
            assign_public_ip: true,
            assign_ipv6_ip: region_network.subnet_has_ipv6.then_some(true),
        },
        metadata: HashMap::from([("user_data".to_string(), BASE64.encode(startup_script))]),
        freeform_tags,
    })
}

async fn get_first_availability_domain(client: &OciClient, region: &str) -> Result<String> {
    let url = client.build_identity_url(&format!(
        "/availabilityDomains?compartmentId={}",
        client.get_compartment_ocid()
    ));
    let domains: Vec<AvailabilityDomainResponse> = client.get(&url).await?;
    domains
        .into_iter()
        .next()
        .map(|domain| domain.name)
        .ok_or(OracleError::NoAvailabilityDomains {
            region: region.to_string(),
        })
}

async fn get_instance(client: &OciClient, instance_ocid: &str) -> Result<InstanceResponse> {
    client
        .get(&client.build_core_url(&format!("/instances/{instance_ocid}")))
        .await
}

enum InstanceWaitError {
    NotRunningYet,
    LookupFailed(OracleError),
    Terminal(OracleError),
}

async fn wait_until_running(client: &OciClient, instance_ocid: &str) -> Result<()> {
    let mut consecutive_lookup_failures: u32 = 0;
    let outcome = retry_if(
        || async move {
            let details = get_instance(client, instance_ocid)
                .await
                .map_err(InstanceWaitError::LookupFailed)?;
            match details.lifecycle_state.as_str() {
                "RUNNING" => {
                    debug!("[OCI] Instance {instance_ocid} reached RUNNING state");
                    Ok(())
                }
                "TERMINATED" | "TERMINATING" => Err(InstanceWaitError::Terminal(
                    OracleError::InstanceTerminated {
                        instance_ocid: instance_ocid.to_string(),
                        lifecycle_state: details.lifecycle_state,
                    },
                )),
                _ => Err(InstanceWaitError::NotRunningYet),
            }
        },
        INSTANCE_RUNNING_POLL_POLICY,
        |error| match error {
            InstanceWaitError::NotRunningYet => {
                consecutive_lookup_failures = 0;
                true
            }
            InstanceWaitError::LookupFailed(lookup_error) => {
                consecutive_lookup_failures += 1;
                warn!(
                    "[OCI] Instance {instance_ocid} not yet visible (attempt {consecutive_lookup_failures}/{MAX_CONSECUTIVE_LOOKUP_FAILURES}): {lookup_error}"
                );
                consecutive_lookup_failures <= MAX_CONSECUTIVE_LOOKUP_FAILURES
            }
            InstanceWaitError::Terminal(_) => false,
        },
    )
    .await;

    match outcome {
        Ok(()) => Ok(()),
        Err(InstanceWaitError::NotRunningYet) => Err(OracleError::InstanceNotRunning {
            instance_ocid: instance_ocid.to_string(),
        }),
        Err(InstanceWaitError::LookupFailed(error) | InstanceWaitError::Terminal(error)) => {
            Err(error)
        }
    }
}

async fn get_public_ips(client: &OciClient, instance_ocid: &str) -> (String, String) {
    let attachments_url = client.build_core_url(&format!(
        "/vnicAttachments?compartmentId={}&instanceId={instance_ocid}",
        client.get_compartment_ocid()
    ));
    let Ok(attachments): Result<Vec<VnicAttachmentResponse>> = client.get(&attachments_url).await
    else {
        warn!("[OCI] Failed to fetch VNIC attachments for instance {instance_ocid}");
        return (String::new(), String::new());
    };
    let Some(vnic_ocid) = attachments
        .into_iter()
        .next()
        .map(|attachment| attachment.vnic_id)
    else {
        warn!("[OCI] No VNIC attachment found for instance {instance_ocid}");
        return (String::new(), String::new());
    };

    let vnic_url = client.build_core_url(&format!("/vnics/{vnic_ocid}"));
    let Ok(vnic): Result<VnicResponse> = client.get(&vnic_url).await else {
        warn!("[OCI] Failed to fetch VNIC {vnic_ocid} for instance {instance_ocid}");
        return (String::new(), String::new());
    };
    let public_ip_v4 = vnic.public_ip.unwrap_or_default();
    let public_ip_v6 = vnic
        .ipv6_addresses
        .and_then(|addresses| addresses.into_iter().next())
        .unwrap_or_default();
    (public_ip_v4, public_ip_v6)
}

fn is_managed_instance(instance: &InstanceResponse) -> bool {
    let has_managed_name = instance
        .display_name
        .as_deref()
        .is_some_and(|name| name.contains(MANAGED_BY_TAG_VALUE));
    let has_managed_tag = instance
        .freeform_tags
        .as_ref()
        .and_then(|tags| tags.get(MANAGED_BY_TAG_KEY))
        .is_some_and(|value| value == MANAGED_BY_TAG_VALUE);
    has_managed_name || has_managed_tag
}

fn build_instance_info(instance: &InstanceResponse, region: &str) -> InstanceInfo {
    InstanceInfo {
        id: instance.id.clone(),
        name: instance.display_name.clone(),
        region: region.to_string(),
        state: OciLifecycleState::from_state_name(&instance.lifecycle_state).into(),
        error_reason: None,
        public_ip_v4: String::new(),
        public_ip_v6: String::new(),
        provider: CloudProviderName::Oracle,
        instance_type: instance.shape.clone(),
        launched_at: instance
            .time_created
            .as_deref()
            .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
            .map(|datetime| datetime.with_timezone(&Utc)),
        spawn_id: instance
            .freeform_tags
            .as_ref()
            .and_then(|tags| tags.get(SPAWN_ID_TAG_KEY))
            .cloned(),
    }
}
