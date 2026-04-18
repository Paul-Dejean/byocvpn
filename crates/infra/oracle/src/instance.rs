use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, SpawnInstanceParams},
    error::{ComputeProvisioningError, Error as CoreError, Result},
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use crate::{
    client::OciClient, startup_script::generate_server_startup_script, state::OciLifecycleState,
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

    let mut create_vnic_details = json!({
        "subnetId": subnet_ocid,
        "assignPublicIp": true,
    });
    if subnet_has_ipv6 {
        create_vnic_details["assignIpv6Ip"] = json!(true);
    }

    let body = json!({
        "compartmentId": compartment_ocid,
        "displayName": INSTANCE_DISPLAY_NAME,
        "availabilityDomain": get_first_availability_domain(client, compartment_ocid).await?,
        "shape": INSTANCE_SHAPE,
        "shapeConfig": {
            "ocpus": INSTANCE_OCPUS,
            "memoryInGBs": INSTANCE_MEMORY_GB,
        },
        "sourceDetails": {
            "sourceType": "image",
            "imageId": image_ocid,
        },
        "createVnicDetails": create_vnic_details,
        "metadata": {
            "user_data": encoded_user_data,
        },
        "freeformTags": {
            "created-by": "byocvpn",
        },
    });

    debug!(
        "Launching OCI instance in {} (compartment {}, subnet {})",
        region, compartment_ocid, subnet_ocid
    );
    let url = format!("{}/20160918/instances", client.build_core_base_url());
    let response = client.post(&url, &body).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    let instance_ocid = response["id"]
        .as_str()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?
        .to_string();

    debug!(
        "OCI instance {} created, waiting for RUNNING state",
        instance_ocid
    );
    wait_until_running(client, &instance_ocid, region).await?;

    let details = get_instance_details(client, &instance_ocid).await?;
    let mut info = build_instance_info(&details, region)?;
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
    let url = format!(
        "{}/20160918/instances/{}?preserveBootVolume=false",
        client.build_core_base_url(),
        instance_ocid
    );
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
    let url = format!(
        "{}/20160918/instances?compartmentId={}&lifecycleState=RUNNING",
        client.build_core_base_url(),
        compartment_ocid
    );
    let response =
        client
            .get(&url)
            .await
            .map_err(|error| ComputeProvisioningError::InstanceSpawnFailed {
                region_name: region.to_string(),
                reason: error.to_string(),
            })?;

    let mut instances: Vec<InstanceInfo> = response
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|instance| {
            instance["displayName"]
                .as_str()
                .map(|name| name.contains("byocvpn"))
                .unwrap_or(false)
                || instance["freeformTags"]["created-by"]
                    .as_str()
                    .map(|tag_value| tag_value == "byocvpn")
                    .unwrap_or(false)
        })
        .filter_map(|instance| build_instance_info(&instance, region).ok())
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
    let url = format!(
        "{}/20160918/availabilityDomains?compartmentId={}",
        client.build_identity_base_url(),
        compartment_ocid
    );
    let response = client.get(&url).await?;
    response
        .as_array()
        .and_then(|availability_domains| availability_domains.first())
        .and_then(|availability_domain| availability_domain["name"].as_str())
        .map(|name| name.to_string())
        .ok_or_else(|| {
            ComputeProvisioningError::InstanceSpawnFailed {
                region_name: "unknown".to_string(),
                reason: "No availability domains found".to_string(),
            }
            .into()
        })
}

async fn get_instance_details(client: &OciClient, instance_ocid: &str) -> Result<Value> {
    let url = format!(
        "{}/20160918/instances/{}",
        client.build_core_base_url(),
        instance_ocid
    );
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
                let state = details["lifecycleState"].as_str().unwrap_or("");
                match state {
                    "RUNNING" => {
                        debug!("Instance {} reached RUNNING state", instance_ocid);
                        return Ok(());
                    }
                    "TERMINATED" | "TERMINATING" => {
                        return Err(ComputeProvisioningError::InstanceWaitFailed {
                            reason: format!(
                                "Instance {} entered terminal state '{}'",
                                instance_ocid, state
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
    let vnic_url = format!(
        "{}/20160918/vnicAttachments?compartmentId={}&instanceId={}",
        client.build_core_base_url(),
        compartment_ocid,
        instance_ocid
    );
    let Ok(vnic_attachments) = client.get(&vnic_url).await else {
        warn!(
            "Failed to fetch VNIC attachments for instance {}",
            instance_ocid
        );
        return (String::new(), String::new());
    };
    let vnic_ocid = vnic_attachments
        .as_array()
        .and_then(|list| list.first())
        .and_then(|attachment| attachment["vnicId"].as_str())
        .unwrap_or_default()
        .to_string();
    if vnic_ocid.is_empty() {
        warn!("No VNIC attachment found for instance {}", instance_ocid);
        return (String::new(), String::new());
    }

    let vnic_url = format!(
        "{}/20160918/vnics/{}",
        client.build_core_base_url(),
        vnic_ocid
    );
    let Ok(vnic) = client.get(&vnic_url).await else {
        warn!(
            "Failed to fetch VNIC details for {} (instance {})",
            vnic_ocid, instance_ocid
        );
        return (String::new(), String::new());
    };
    let public_ip_v4 = vnic["publicIp"].as_str().unwrap_or_default().to_string();
    let public_ip_v6 = vnic["ipv6Addresses"]
        .as_array()
        .and_then(|list| list.first())
        .and_then(|address| address.as_str())
        .unwrap_or_default()
        .to_string();
    (public_ip_v4, public_ip_v6)
}

fn build_instance_info(instance: &Value, region: &str) -> Result<InstanceInfo> {
    let id = instance["id"]
        .as_str()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?
        .to_string();

    let name = instance["displayName"]
        .as_str()
        .map(|display_name| display_name.to_string());

    let state: InstanceState =
        OciLifecycleState::from(instance["lifecycleState"].as_str().unwrap_or("UNKNOWN")).into();

    let public_ip_v4 = instance["primaryPublicIp"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let public_ip_v6 = instance["primaryIpv6Address"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    Ok(InstanceInfo {
        id,
        name,
        state,
        public_ip_v4,
        public_ip_v6,
        region: region.to_string(),
        provider: CloudProviderName::Oracle,
        instance_type: instance["shape"].as_str().unwrap_or_default().to_string(),
        launched_at: instance["timeCreated"]
            .as_str()
            .and_then(|timestamp| DateTime::parse_from_rfc3339(timestamp).ok())
            .map(|datetime| datetime.with_timezone(&Utc)),
    })
}
