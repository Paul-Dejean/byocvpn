use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byocvpn_core::{
    cloud_provider::{InstanceInfo, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
};
use chrono::Utc;
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use crate::{client::OciClient, cloud_init::generate_wireguard_cloud_init};

/// Shape used for new instances. `VM.Standard.A1.Flex` is part of the OCI Always Free tier.
const INSTANCE_SHAPE: &str = "VM.Standard.A1.Flex";
const INSTANCE_OCPUS: f32 = 1.0;
const INSTANCE_MEMORY_GB: f32 = 6.0;

/// Display name tag applied to every spawned instance.
const INSTANCE_DISPLAY_NAME: &str = "byocvpn-server";

/// Launch a new WireGuard VPN instance in `compartment_ocid`.
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
        generate_wireguard_cloud_init(params.server_private_key, params.client_public_key)?;
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
            "byocvpn": "true",
        },
    });

    let url = format!("{}/20160918/instances", client.core_base_url());
    let response = client.post(&url, &body).await.map_err(|e| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: e.to_string(),
        }
    })?;

    let instance_ocid = response["id"]
        .as_str()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?
        .to_string();

    // Poll until RUNNING (up to 3 minutes)
    wait_until_running(client, &instance_ocid, region).await?;

    // Fetch base instance details, then resolve the public IP via the VNIC API.
    // The GetInstance response never contains the public IP directly — it only
    // appears on the attached VNIC.
    let details = get_instance_details(client, &instance_ocid).await?;
    let mut info = build_instance_info(&details, region)?;
    let (public_ip_v4, public_ip_v6) =
        get_public_ips(client, &instance_ocid, compartment_ocid).await;
    info.public_ip_v4 = public_ip_v4;
    info.public_ip_v6 = public_ip_v6;
    info.instance_type = INSTANCE_SHAPE.to_string();
    info.launched_at = Some(Utc::now());
    Ok(info)
}

/// Terminate (permanently delete) an instance by OCID.
pub async fn terminate_instance(client: &OciClient, instance_ocid: &str) -> Result<()> {
    let url = format!(
        "{}/20160918/instances/{}?preserveBootVolume=false",
        client.core_base_url(),
        instance_ocid
    );
    client.delete(&url).await.map_err(|e| {
        ComputeProvisioningError::InstanceTerminationFailed {
            instance_identifier: instance_ocid.to_string(),
            reason: e.to_string(),
        }
        .into()
    })
}

/// List all running byocvpn instances in `compartment_ocid`.
pub async fn list_instances(
    client: &OciClient,
    compartment_ocid: &str,
    region: &str,
) -> Result<Vec<InstanceInfo>> {
    let url = format!(
        "{}/20160918/instances?compartmentId={}&lifecycleState=RUNNING",
        client.core_base_url(),
        compartment_ocid
    );
    let response =
        client
            .get(&url)
            .await
            .map_err(|e| ComputeProvisioningError::InstanceSpawnFailed {
                region_name: region.to_string(),
                reason: e.to_string(),
            })?;

    let instances = response
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|instance| {
            // Only show instances tagged or named as byocvpn
            instance["displayName"]
                .as_str()
                .map(|name| name.contains("byocvpn"))
                .unwrap_or(false)
                || instance["freeformTags"]["byocvpn"].as_str().is_some()
        })
        .filter_map(|instance| build_instance_info(&instance, region).ok())
        .collect();

    Ok(instances)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// List availability domains for the given compartment.
///
/// NOTE: This is an **Identity** API call (`identity.{region}.oraclecloud.com`),
/// NOT a Core/Compute call (`iaas.{region}.oraclecloud.com`).  Using the wrong
/// base URL returns 404 in newer OCI regions such as eu-paris-1.
async fn get_first_availability_domain(
    client: &OciClient,
    compartment_ocid: &str,
) -> Result<String> {
    let url = format!(
        "{}/20160918/availabilityDomains?compartmentId={}",
        client.identity_base_url(),
        compartment_ocid
    );
    let response = client.get(&url).await?;
    response
        .as_array()
        .and_then(|ads| ads.first())
        .and_then(|ad| ad["name"].as_str())
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
        client.core_base_url(),
        instance_ocid
    );
    client.get(&url).await
}

/// Maximum number of consecutive 404s tolerated while OCI propagates the new instance.
///
/// OCI uses eventual consistency: a freshly-created instance may not yet be
/// queryable immediately after the POST returns.  We treat up to
/// `MAX_NOT_FOUND_RETRIES` consecutive 404 / "not yet visible" responses as
/// transient and keep polling.  A 404 that persists beyond that threshold is
/// re-raised as a real error.
const MAX_NOT_FOUND_RETRIES: u32 = 6;

async fn wait_until_running(client: &OciClient, instance_ocid: &str, _region: &str) -> Result<()> {
    // Give OCI a moment to register the instance before the first query.
    sleep(Duration::from_secs(5)).await;

    let mut not_found_count: u32 = 0;

    for _ in 0..36 {
        // up to ~3 minutes (36 × 5s)
        match get_instance_details(client, instance_ocid).await {
            Ok(details) => {
                not_found_count = 0; // reset on success
                let state = details["lifecycleState"].as_str().unwrap_or("");
                match state {
                    "RUNNING" => return Ok(()),
                    "TERMINATED" | "TERMINATING" => {
                        return Err(ComputeProvisioningError::InstanceWaitFailed {
                            reason: format!(
                                "Instance {} entered terminal state '{}'",
                                instance_ocid, state
                            ),
                        }
                        .into());
                    }
                    _ => {} // PROVISIONING, STARTING, etc. — keep polling
                }
            }
            Err(error) => {
                // OCI eventual consistency: the instance may not be visible
                // for a few seconds after creation.  Allow a small number of
                // consecutive not-found / transient errors before giving up.
                not_found_count += 1;
                if not_found_count > MAX_NOT_FOUND_RETRIES {
                    return Err(error);
                }
                eprintln!(
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

/// Fetch the primary public IPv4 of an instance via its VNIC attachment.
async fn get_public_ips(
    client: &OciClient,
    instance_ocid: &str,
    compartment_ocid: &str,
) -> (String, String) {
    // Get VNIC attachments
    let vnic_url = format!(
        "{}/20160918/vnicAttachments?compartmentId={}&instanceId={}",
        client.core_base_url(),
        compartment_ocid,
        instance_ocid
    );
    let Ok(vnic_attachments) = client.get(&vnic_url).await else {
        return (String::new(), String::new());
    };
    let vnic_ocid = vnic_attachments
        .as_array()
        .and_then(|list| list.first())
        .and_then(|v| v["vnicId"].as_str())
        .unwrap_or_default()
        .to_string();
    if vnic_ocid.is_empty() {
        return (String::new(), String::new());
    }
    // Get VNIC details
    let vnic_url = format!("{}/20160918/vnics/{}", client.core_base_url(), vnic_ocid);
    let Ok(vnic) = client.get(&vnic_url).await else {
        return (String::new(), String::new());
    };
    let public_ip_v4 = vnic["publicIp"].as_str().unwrap_or_default().to_string();
    let public_ip_v6 = vnic["ipv6Addresses"]
        .as_array()
        .and_then(|list| list.first())
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    (public_ip_v4, public_ip_v6)
}

fn build_instance_info(instance: &Value, region: &str) -> Result<InstanceInfo> {
    let id = instance["id"]
        .as_str()
        .ok_or(ComputeProvisioningError::MissingInstanceIdentifier)?
        .to_string();

    let name = instance["displayName"].as_str().map(|s| s.to_string());

    let state = instance["lifecycleState"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_lowercase();

    // The public IPs are not embedded directly in the instance response; they come via the VNIC.
    // We store empty strings here and fetch them properly in spawn_instance after polling.
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
        provider: "oracle".to_string(),
        instance_type: instance["shape"].as_str().unwrap_or_default().to_string(),
        launched_at: instance["timeCreated"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
    })
}
