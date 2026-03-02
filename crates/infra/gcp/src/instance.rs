use byocvpn_core::{
    cloud_provider::{InstanceInfo, SpawnInstanceParams},
    error::{ComputeProvisioningError, Result},
};
use serde_json::json;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

use crate::{
    client::GcpClient, cloud_init::generate_wireguard_startup_script,
    network::primary_zone_for_region,
};

const MACHINE_TYPE: &str = "e2-micro";
const INSTANCE_LABEL: &str = "byocvpn";
const INSTANCE_TAG: &str = "byocvpn";

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Launch a new byocvpn WireGuard VM in `region`.
pub async fn spawn_instance(
    client: &GcpClient,
    subnet_self_link: &str,
    image_self_link: &str,
    region: &str,
    params: &SpawnInstanceParams<'_>,
) -> Result<InstanceInfo> {
    let zone = primary_zone_for_region(region);
    let startup_script =
        generate_wireguard_startup_script(params.server_private_key, params.client_public_key)?;

    // Instance names must be lowercase letters, digits, and hyphens, max 63 chars.
    let instance_name = format!(
        "byocvpn-{}",
        Uuid::new_v4().to_string().replace('-', "")[..12].to_lowercase()
    );

    let machine_type_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/zones/{}/machineTypes/{}",
        client.project_id, zone, MACHINE_TYPE
    );

    let body = json!({
        "name": instance_name,
        "machineType": machine_type_url,
        "disks": [{
            "boot": true,
            "autoDelete": true,
            "initializeParams": {
                "sourceImage": image_self_link,
                "diskSizeGb": "10",
            }
        }],
        "networkInterfaces": [{
            "subnetwork": subnet_self_link,
            "stackType": "IPV4_IPV6",
            "accessConfigs": [{
                "type": "ONE_TO_ONE_NAT",
                "name": "External NAT",
                "networkTier": "PREMIUM",
            }],
            "ipv6AccessConfigs": [{
                "type": "DIRECT_IPV6",
                "name": "External IPv6",
                "networkTier": "PREMIUM",
            }]
        }],
        "metadata": {
            "items": [
                { "key": "startup-script", "value": startup_script }
            ]
        },
        "labels": { INSTANCE_LABEL: "true" },
        "tags": { "items": [INSTANCE_TAG] },
    });

    let url = format!("{}/zones/{}/instances", client.compute_base_url(), zone);
    let operation = client.post(&url, &body).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    // Poll the zone operation until the instance exists.
    let operation_url = operation["selfLink"]
        .as_str()
        .ok_or_else(|| ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: "Missing selfLink in operation response".to_string(),
        })?
        .to_string();
    wait_for_zone_operation(client, &operation_url, region).await?;

    // Fetch the instance to get its public IP.
    let instance = get_instance(client, &zone, &instance_name).await?;
    let public_ip_v4 = extract_public_ip_v4(&instance);
    let public_ip_v6 = extract_public_ip_v6(&instance);

    let id = format!("{}/{}", zone, instance_name);
    Ok(InstanceInfo {
        id,
        name: Some(instance_name),
        region: region.to_string(),
        state: "RUNNING".to_string(),
        public_ip_v4,
        public_ip_v6,
        provider: "gcp".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Terminate
// ---------------------------------------------------------------------------

/// Delete a GCP instance. `instance_id` is `{zone}/{instance_name}`.
pub async fn terminate_instance(client: &GcpClient, instance_id: &str) -> Result<()> {
    let (zone, instance_name) = parse_instance_id(instance_id)?;
    let url = format!(
        "{}/zones/{}/instances/{}",
        client.compute_base_url(),
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

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// List all running byocvpn instances across all zones in `region`.
pub async fn list_instances(client: &GcpClient, region: &str) -> Result<Vec<InstanceInfo>> {
    // aggregatedList scoped to a region via a filter is the most efficient path.
    let url = format!(
        "{}/aggregated/instances?filter=labels.{region_label}%3Dtrue&maxResults=500",
        client.compute_base_url(),
        region_label = INSTANCE_LABEL,
    );
    let response = client.get(&url).await?;

    let mut instances = Vec::new();
    if let Some(items) = response["items"].as_object() {
        for (zone_key, zone_data) in items {
            // zone_key looks like "zones/us-central1-a"
            if !zone_key.starts_with(&format!("zones/{}", region)) {
                continue;
            }
            let zone = zone_key.trim_start_matches("zones/");
            if let Some(instance_list) = zone_data["instances"].as_array() {
                for instance in instance_list {
                    if let Some(info) = parse_instance_info(instance, zone, region) {
                        instances.push(info);
                    }
                }
            }
        }
    }
    Ok(instances)
}

/// List all running byocvpn instances across every region/zone.
pub async fn list_all_instances(client: &GcpClient) -> Result<Vec<InstanceInfo>> {
    let url = format!(
        "{}/aggregated/instances?filter=labels.{region_label}%3Dtrue&maxResults=500",
        client.compute_base_url(),
        region_label = INSTANCE_LABEL,
    );
    let response = client.get(&url).await?;

    let mut instances = Vec::new();
    if let Some(items) = response["items"].as_object() {
        for (zone_key, zone_data) in items {
            let zone = zone_key.trim_start_matches("zones/");
            // Derive region from zone (strip trailing -[a-z]).
            let region = zone_to_region(zone);
            if let Some(instance_list) = zone_data["instances"].as_array() {
                for instance in instance_list {
                    if let Some(info) = parse_instance_info(instance, zone, &region) {
                        instances.push(info);
                    }
                }
            }
        }
    }
    Ok(instances)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
) -> Result<serde_json::Value> {
    let url = format!(
        "{}/zones/{}/instances/{}",
        client.compute_base_url(),
        zone,
        instance_name
    );
    client.get(&url).await
}

fn extract_public_ip_v4(instance: &serde_json::Value) -> String {
    instance["networkInterfaces"]
        .as_array()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface["accessConfigs"].as_array())
        .and_then(|configs| configs.first())
        .and_then(|config| config["natIP"].as_str())
        .unwrap_or_default()
        .to_string()
}

fn extract_public_ip_v6(instance: &serde_json::Value) -> String {
    instance["networkInterfaces"]
        .as_array()
        .and_then(|interfaces| interfaces.first())
        .and_then(|interface| interface["ipv6AccessConfigs"].as_array())
        .and_then(|configs| configs.first())
        .and_then(|config| config["externalIpv6"].as_str())
        .unwrap_or_default()
        .to_string()
}

fn zone_to_region(zone: &str) -> String {
    // e.g. "us-central1-a" → "us-central1"
    let mut parts: Vec<&str> = zone.split('-').collect();
    if parts.len() > 2 {
        parts.pop(); // remove trailing zone letter
    }
    parts.join("-")
}

fn parse_instance_info(
    instance: &serde_json::Value,
    zone: &str,
    region: &str,
) -> Option<InstanceInfo> {
    let name = instance["name"].as_str()?.to_string();
    let status = instance["status"].as_str().unwrap_or("UNKNOWN").to_string();
    let public_ip_v4 = extract_public_ip_v4(instance);
    let public_ip_v6 = extract_public_ip_v6(instance);
    let id = format!("{}/{}", zone, name);
    Some(InstanceInfo {
        id,
        name: Some(name),
        region: region.to_string(),
        state: status,
        public_ip_v4,
        public_ip_v6,
        provider: "gcp".to_string(),
    })
}

async fn wait_for_zone_operation(
    client: &GcpClient,
    operation_url: &str,
    region: &str,
) -> Result<()> {
    for attempt in 1..=60u32 {
        let operation = client.get(operation_url).await?;
        match operation["status"].as_str() {
            Some("DONE") => {
                if let Some(error) = operation.get("error") {
                    let message = error["errors"]
                        .as_array()
                        .and_then(|errors| errors.first())
                        .and_then(|error| error["message"].as_str())
                        .unwrap_or("unknown error");
                    return Err(ComputeProvisioningError::InstanceSpawnFailed {
                        region_name: region.to_string(),
                        reason: message.to_string(),
                    }
                    .into());
                }
                return Ok(());
            }
            _ => {
                eprintln!(
                    "[GCP] Waiting for instance creation (attempt {}/60)...",
                    attempt
                );
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
    Err(ComputeProvisioningError::InstanceSpawnFailed {
        region_name: region.to_string(),
        reason: "Timed out waiting for instance creation operation".to_string(),
    }
    .into())
}
