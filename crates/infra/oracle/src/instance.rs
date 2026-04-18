use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, SpawnInstanceParams},
    error::{ComputeProvisioningError, Error as CoreError, Result},
};
use chrono::{DateTime, Utc};
use tokio::time::{Duration, sleep};

use crate::{
    client::OciClient,
    models::{
        AvailabilityDomain, CreateVnicDetails, InstanceResponse, LaunchInstanceRequest,
        ShapeConfig, SourceDetails, Vnic, VnicAttachment, byocvpn_tags,
    },
    startup_script::generate_server_startup_script,
    state::OciLifecycleState,
};
use log::*;

const INSTANCE_SHAPE: &str = "VM.Standard.A1.Flex";
const INSTANCE_OCPUS: f32 = 1.0;
const INSTANCE_MEMORY_GB: f32 = 6.0;
const INSTANCE_DISPLAY_NAME: &str = "byocvpn-server";

pub async fn spawn_instance(
    client: &OciClient,
    compartment_ocid: &str,
    subnet_ocid: &str,
    subnet_has_ipv6: bool,
    image_ocid: &str,
    region: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let user_data =
        generate_server_startup_script(params.server_private_key, params.client_public_key)?;
    let encoded_user_data = BASE64.encode(&user_data);

    let body = LaunchInstanceRequest {
        compartment_id: compartment_ocid.to_string(),
        display_name: INSTANCE_DISPLAY_NAME.to_string(),
        availability_domain: get_first_availability_domain(client, compartment_ocid).await?,
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
            subnet_id: subnet_ocid.to_string(),
            assign_public_ip: true,
            assign_ipv6_ip: subnet_has_ipv6.then_some(true),
        },
        metadata: HashMap::from([("user_data".to_string(), encoded_user_data)]),
        freeform_tags: byocvpn_tags(),
    };

    debug!(
        "Launching OCI instance in {} (compartment {}, subnet {})",
        region, compartment_ocid, subnet_ocid
    );
    let url = client.build_core_url("/instances");
    let response: InstanceResponse = client.post(&url, &body).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    let instance_ocid = response.id.clone();
    debug!(
        "OCI instance {} created, waiting for RUNNING state",
        instance_ocid
    );
    wait_until_running(client, &instance_ocid, region).await?;

    let details = get_instance_details(client, &instance_ocid).await?;
    let mut info = build_instance_info(&details, region);
    let (public_ip_v4, public_ip_v6) =
        get_public_ips(client, &instance_ocid, compartment_ocid).await;
    info.public_ip_v4 = public_ip_v4;
    info.public_ip_v6 = public_ip_v6;
    info.instance_type = INSTANCE_SHAPE.to_string();
    info.launched_at = Some(Utc::now());
    info!(
        "OCI instance {} spawned in {} — IPv4: {}, IPv6: {}",
        info.id, region, info.public_ip_v4, info.public_ip_v6
    );
    Ok(info)
}

pub async fn terminate_instance(client: &OciClient, instance_ocid: &str) -> Result<()> {
    debug!("Terminating OCI instance {}", instance_ocid);
    let url = client.build_core_url(&format!("/instances/{}?preserveBootVolume=false", instance_ocid));
    client.delete(&url).await.map_err(|error| {
        CoreError::from(ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_ocid.to_string(),
            reason: error.to_string(),
        })
    })?;
    info!("OCI instance {} terminated", instance_ocid);
    Ok(())
}

pub async fn list_instances(
    client: &OciClient,
    compartment_ocid: &str,
    region: &str,
) -> Result<Vec<InstanceInfo>> {
    let url = client.build_core_url(&format!("/instances?compartmentId={}&lifecycleState=RUNNING", compartment_ocid));
    let response: Vec<InstanceResponse> =
        client
            .get(&url)
            .await
            .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
                region_name: region.to_string(),
                reason: error.to_string(),
            })?;

    let mut instances: Vec<InstanceInfo> = response
        .into_iter()
        .filter(|instance| {
            instance
                .display_name
                .as_deref()
                .map(|name| name.contains("byocvpn"))
                .unwrap_or(false)
                || instance
                    .freeform_tags
                    .as_ref()
                    .and_then(|tags| tags.get("created-by"))
                    .map(|value| value == "byocvpn")
                    .unwrap_or(false)
        })
        .map(|instance| build_instance_info(&instance, region))
        .collect();

    let ip_results = futures::future::join_all(
        instances
            .iter()
            .map(|instance| get_public_ips(client, &instance.id, compartment_ocid)),
    )
    .await;

    for (instance, (public_ip_v4, public_ip_v6)) in instances.iter_mut().zip(ip_results) {
        instance.public_ip_v4 = public_ip_v4;
        instance.public_ip_v6 = public_ip_v6;
    }

    Ok(instances)
}

async fn get_first_availability_domain(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<String> {
    let url = client.build_identity_url(&format!("/availabilityDomains?compartmentId={}", compartment_ocid));
    let domains: Vec<AvailabilityDomain> = client.get(&url).await?;
    domains
        .into_iter()
        .next()
        .map(|domain| domain.name)
        .ok_or_else(|| {
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: "unknown".to_string(),
                reason: "No availability domains found".to_string(),
            }
            .into()
        })
}

async fn get_instance_details(client: &OciClient, instance_ocid: &str) -> Result<InstanceResponse> {
    let url = client.build_core_url(&format!("/instances/{}", instance_ocid));
    client.get(&url).await
}

const MAX_NOT_FOUND_RETRIES: u32 = 6;

async fn wait_until_running(client: &OciClient, instance_ocid: &str, _region: &str) -> Result<()> {
    sleep(Duration::from_secs(5)).await;

    let mut not_found_count: u32 = 0;

    for _ in 0..36 {
        match get_instance_details(client, instance_ocid).await {
            Ok(details) => {
                not_found_count = 0;
                match details.lifecycle_state.as_str() {
                    "RUNNING" => {
                        debug!("Instance {} reached RUNNING state", instance_ocid);
                        return Ok(());
                    }
                    "TERMINATED" | "TERMINATING" => {
                        return Err(ComputeProvisioningError::InstanceWaitFailed {
                            reason: format!(
                                "Instance {} entered terminal state '{}'",
                                instance_ocid, details.lifecycle_state
                            ),
                        }
                        .into());
                    }
                    _ => {}
                }
            }
            Err(error) => {
                not_found_count += 1;
                if not_found_count > MAX_NOT_FOUND_RETRIES {
                    return Err(error);
                }
                warn!(
                    "Instance {} not yet visible (attempt {}/{}): {}",
                    instance_ocid, not_found_count, MAX_NOT_FOUND_RETRIES, error
                );
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
    Err(ComputeProvisioningError::InstanceWaitFailed {
        reason: format!("Instance {} did not become RUNNING in time", instance_ocid),
    }
    .into())
}

async fn get_public_ips(
    client: &OciClient,
    instance_ocid: &str,
    compartment_ocid: &str,
) -> (String, String) {
    let vnic_url = client.build_core_url(&format!("/vnicAttachments?compartmentId={}&instanceId={}", compartment_ocid, instance_ocid));
    let Ok(vnic_attachments): Result<Vec<VnicAttachment>> = client.get(&vnic_url).await else {
        warn!(
            "Failed to fetch VNIC attachments for instance {}",
            instance_ocid
        );
        return (String::new(), String::new());
    };
    let Some(vnic_ocid) = vnic_attachments.into_iter().next().map(|attachment| attachment.vnic_id) else {
        warn!("No VNIC attachment found for instance {}", instance_ocid);
        return (String::new(), String::new());
    };

    let vnic_url = client.build_core_url(&format!("/vnics/{}", vnic_ocid));
    let Ok(vnic): Result<Vnic> = client.get(&vnic_url).await else {
        warn!(
            "Failed to fetch VNIC details for {} (instance {})",
            vnic_ocid, instance_ocid
        );
        return (String::new(), String::new());
    };
    let public_ip_v4 = vnic.public_ip.unwrap_or_default();
    let public_ip_v6 = vnic
        .ipv6_addresses
        .and_then(|addresses| addresses.into_iter().next())
        .unwrap_or_default();
    (public_ip_v4, public_ip_v6)
}

fn build_instance_info(instance: &InstanceResponse, region: &str) -> InstanceInfo {
    InstanceInfo {
        id: instance.id.clone(),
        name: instance.display_name.clone(),
        state: OciLifecycleState::from(instance.lifecycle_state.as_str()).into(),
        public_ip_v4: String::new(),
        public_ip_v6: String::new(),
        region: region.to_string(),
        provider: CloudProviderName::Oracle,
        instance_type: instance.shape.clone(),
        launched_at: instance
            .time_created
            .as_deref()
            .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
            .map(|datetime| datetime.with_timezone(&Utc)),
    }
}
