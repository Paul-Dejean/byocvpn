use byocvpn_core::error::{ConfigurationError, NetworkProvisioningError, Result};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use crate::client::GcpClient;

const VPC_NAME: &str = "byocvpn-vpc";
const SUBNET_NAME: &str = "byocvpn-subnet";
const FIREWALL_WIREGUARD_IPV4_NAME: &str = "byocvpn-wireguard-ipv4";
const FIREWALL_WIREGUARD_IPV6_NAME: &str = "byocvpn-wireguard-ipv6";
const FIREWALL_HEALTH_IPV4_NAME: &str = "byocvpn-health-ipv4";
const FIREWALL_HEALTH_IPV6_NAME: &str = "byocvpn-health-ipv6";
const FIREWALL_TAG: &str = "byocvpn";

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Poll a GCP long-running operation until it reaches `DONE` (or times out).
async fn wait_for_operation(client: &GcpClient, operation_url: &str) -> Result<()> {
    for attempt in 1..=60u32 {
        let operation = client.get(operation_url).await?;
        match operation["status"].as_str() {
            Some("DONE") => {
                // Surface any embedded error.
                if let Some(error) = operation.get("error") {
                    let message = error["errors"]
                        .as_array()
                        .and_then(|errors| errors.first())
                        .and_then(|error| error["message"].as_str())
                        .unwrap_or("unknown error");
                    return Err(ConfigurationError::InvalidCloudProvider(format!(
                        "GCP operation failed: {}",
                        message
                    ))
                    .into());
                }
                return Ok(());
            }
            Some("PENDING") | Some("RUNNING") => {
                eprintln!(
                    "[GCP] operation pending (attempt {}/60): {}",
                    attempt, operation_url
                );
                sleep(Duration::from_secs(3)).await;
            }
            other => {
                return Err(ConfigurationError::InvalidCloudProvider(format!(
                    "Unexpected GCP operation status: {:?}",
                    other
                ))
                .into());
            }
        }
    }
    Err(ConfigurationError::InvalidCloudProvider(format!(
        "Timed out waiting for GCP operation: {}",
        operation_url
    ))
    .into())
}

/// Extract the `selfLink` from an operation response and poll it to completion.
async fn wait_for_operation_response(client: &GcpClient, response: &Value) -> Result<()> {
    let operation_url = response["selfLink"]
        .as_str()
        .ok_or_else(|| {
            ConfigurationError::InvalidCloudProvider(
                "GCP operation response missing selfLink".to_string(),
            )
        })?
        .to_string();
    wait_for_operation(client, &operation_url).await
}

// ---------------------------------------------------------------------------
// VPC (Global Network)
// ---------------------------------------------------------------------------

/// Ensure the byocvpn VPC exists, creating it if necessary. Returns the VPC self-link.
pub async fn get_or_create_vpc(client: &GcpClient) -> Result<String> {
    let url = format!("{}/global/networks/{}", client.compute_base_url(), VPC_NAME);

    match client.get(&url).await {
        Ok(existing) => {
            return Ok(existing["selfLink"]
                .as_str()
                .unwrap_or_default()
                .to_string());
        }
        Err(_) => {}
    }

    let create_url = format!("{}/global/networks", client.compute_base_url());
    let body = json!({
        "name": VPC_NAME,
        "autoCreateSubnetworks": false,
        "description": "byocvpn WireGuard VPN network",
    });
    let operation = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;

    // Fetch the newly created VPC.
    let vpc = client.get(&url).await?;
    println!("GCP VPC '{}' created.", VPC_NAME);
    Ok(vpc["selfLink"].as_str().unwrap_or_default().to_string())
}

// ---------------------------------------------------------------------------
// Firewall Rules (Global)
// ---------------------------------------------------------------------------

/// Ensure firewall rules allowing UDP 51820 inbound exist on the byocvpn VPC.
/// GCP requires separate rules for IPv4 and IPv6 — mixed sourceRanges are rejected.
pub async fn get_or_create_firewall(client: &GcpClient) -> Result<()> {
    let vpc_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/global/networks/{}",
        client.project_id, VPC_NAME
    );
    let create_url = format!("{}/global/firewalls", client.compute_base_url());

    // IPv4 rule
    let ipv4_url = format!(
        "{}/global/firewalls/{}",
        client.compute_base_url(),
        FIREWALL_WIREGUARD_IPV4_NAME
    );
    if client.get(&ipv4_url).await.is_err() {
        let body = json!({
            "name": FIREWALL_WIREGUARD_IPV4_NAME,
            "network": vpc_url,
            "description": "Allow WireGuard UDP 51820 inbound (IPv4) for byocvpn",
            "direction": "INGRESS",
            "priority": 1000,
            "targetTags": [FIREWALL_TAG],
            "allowed": [{ "IPProtocol": "udp", "ports": ["51820"] }],
            "sourceRanges": ["0.0.0.0/0"],
        });
        let operation = client.post(&create_url, &body).await.map_err(|error| {
            NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            }
        })?;
        wait_for_operation_response(client, &operation).await?;
        println!(
            "GCP firewall rule '{}' created.",
            FIREWALL_WIREGUARD_IPV4_NAME
        );
    }

    // IPv6 rule
    let ipv6_url = format!(
        "{}/global/firewalls/{}",
        client.compute_base_url(),
        FIREWALL_WIREGUARD_IPV6_NAME
    );
    if client.get(&ipv6_url).await.is_err() {
        let body = json!({
            "name": FIREWALL_WIREGUARD_IPV6_NAME,
            "network": vpc_url,
            "description": "Allow WireGuard UDP 51820 inbound (IPv6) for byocvpn",
            "direction": "INGRESS",
            "priority": 1000,
            "targetTags": [FIREWALL_TAG],
            "allowed": [{ "IPProtocol": "udp", "ports": ["51820"] }],
            "sourceRanges": ["::/0"],
        });
        let operation = client.post(&create_url, &body).await.map_err(|error| {
            NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            }
        })?;
        wait_for_operation_response(client, &operation).await?;
        println!(
            "GCP firewall rule '{}' created.",
            FIREWALL_WIREGUARD_IPV6_NAME
        );
    }

    // Health endpoint IPv4 rule (TCP 51820).
    let health_ipv4_url = format!(
        "{}/global/firewalls/{}",
        client.compute_base_url(),
        FIREWALL_HEALTH_IPV4_NAME
    );
    if client.get(&health_ipv4_url).await.is_err() {
        let body = json!({
            "name": FIREWALL_HEALTH_IPV4_NAME,
            "network": vpc_url,
            "description": "Allow health endpoint TCP 51820 inbound (IPv4) for byocvpn",
            "direction": "INGRESS",
            "priority": 1000,
            "targetTags": [FIREWALL_TAG],
            "allowed": [{ "IPProtocol": "tcp", "ports": ["51820"] }],
            "sourceRanges": ["0.0.0.0/0"],
        });
        let operation = client.post(&create_url, &body).await.map_err(|error| {
            NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            }
        })?;
        wait_for_operation_response(client, &operation).await?;
        println!("GCP firewall rule '{}' created.", FIREWALL_HEALTH_IPV4_NAME);
    }

    // Health endpoint IPv6 rule (TCP 51820).
    let health_ipv6_url = format!(
        "{}/global/firewalls/{}",
        client.compute_base_url(),
        FIREWALL_HEALTH_IPV6_NAME
    );
    if client.get(&health_ipv6_url).await.is_err() {
        let body = json!({
            "name": FIREWALL_HEALTH_IPV6_NAME,
            "network": vpc_url,
            "description": "Allow health endpoint TCP 51820 inbound (IPv6) for byocvpn",
            "direction": "INGRESS",
            "priority": 1000,
            "targetTags": [FIREWALL_TAG],
            "allowed": [{ "IPProtocol": "tcp", "ports": ["51820"] }],
            "sourceRanges": ["::/0"],
        });
        let operation = client.post(&create_url, &body).await.map_err(|error| {
            NetworkProvisioningError::SecurityGroupCreationFailed {
                reason: error.to_string(),
            }
        })?;
        wait_for_operation_response(client, &operation).await?;
        println!("GCP firewall rule '{}' created.", FIREWALL_HEALTH_IPV6_NAME);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Subnets (Regional)
// ---------------------------------------------------------------------------

/// Deterministic, non-overlapping `/20` subnet CIDR for `region`.
///
/// Uses a stable lookup table for all known GCP regions. New/unknown regions
/// fall back to a hash-based CIDR in the `10.200.x.0/20` range.
fn subnet_cidr_for_region(region: &str) -> String {
    // Sorted alphabetically; each entry maps to a unique `10.{n}.0.0/20`.
    const REGION_CIDRS: &[(&str, u8)] = &[
        ("africa-south1", 100),
        ("asia-east1", 101),
        ("asia-east2", 102),
        ("asia-northeast1", 103),
        ("asia-northeast2", 104),
        ("asia-northeast3", 105),
        ("asia-south1", 106),
        ("asia-south2", 107),
        ("asia-southeast1", 108),
        ("asia-southeast2", 109),
        ("australia-southeast1", 110),
        ("australia-southeast2", 111),
        ("europe-central2", 112),
        ("europe-north1", 113),
        ("europe-southwest1", 114),
        ("europe-west1", 115),
        ("europe-west10", 116),
        ("europe-west12", 117),
        ("europe-west2", 118),
        ("europe-west3", 119),
        ("europe-west4", 120),
        ("europe-west6", 121),
        ("europe-west8", 122),
        ("europe-west9", 123),
        ("me-central1", 124),
        ("me-central2", 125),
        ("me-west1", 126),
        ("northamerica-northeast1", 127),
        ("northamerica-northeast2", 128),
        ("northamerica-south1", 129),
        ("southamerica-east1", 130),
        ("southamerica-west1", 131),
        ("us-central1", 132),
        ("us-east1", 133),
        ("us-east4", 134),
        ("us-east5", 135),
        ("us-south1", 136),
        ("us-west1", 137),
        ("us-west2", 138),
        ("us-west3", 139),
        ("us-west4", 140),
    ];

    if let Some((_, octet)) = REGION_CIDRS.iter().find(|(name, _)| *name == region) {
        return format!("10.{}.0.0/20", octet);
    }

    // Fallback for future regions: hash the name into 200–240 range.
    let hash: u8 = region
        .bytes()
        .enumerate()
        .fold(0u32, |acc, (index, byte)| {
            acc.wrapping_add((byte as u32).wrapping_mul((index + 1) as u32))
        })
        .wrapping_rem(40) as u8;
    format!("10.{}.0.0/20", 200 + hash)
}

/// Ensure a regional subnet named `byocvpn-subnet` exists. Returns the subnet self-link.
pub async fn get_or_create_subnet(client: &GcpClient, region: &str) -> Result<String> {
    let url = format!(
        "{}/regions/{}/subnetworks/{}",
        client.compute_base_url(),
        region,
        SUBNET_NAME
    );

    if let Ok(existing) = client.get(&url).await {
        let self_link = existing["selfLink"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        // Upgrade IPv4-only subnets to dual-stack so VMs can get an external IPv6 address.
        if existing["stackType"].as_str() != Some("IPV4_IPV6") {
            eprintln!(
                "[GCP] Upgrading subnet '{}' in {} to IPV4_IPV6...",
                SUBNET_NAME, region
            );
            let fingerprint = existing["fingerprint"].as_str().unwrap_or("").to_string();
            let patch_body = json!({
                "stackType": "IPV4_IPV6",
                "ipv6AccessType": "EXTERNAL",
                "fingerprint": fingerprint,
            });
            let operation = client.patch(&url, &patch_body).await.map_err(|error| {
                NetworkProvisioningError::SubnetCreationFailed {
                    reason: format!("Failed to upgrade subnet to IPV4_IPV6: {}", error),
                }
            })?;
            wait_for_operation_response(client, &operation)
                .await
                .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                    reason: format!("Subnet IPV4_IPV6 upgrade operation failed: {}", error),
                })?;
            println!(
                "GCP subnet '{}' upgraded to IPV4_IPV6 in {}.",
                SUBNET_NAME, region
            );
        }

        return Ok(self_link);
    }

    let vpc_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/global/networks/{}",
        client.project_id, VPC_NAME
    );
    let create_url = format!(
        "{}/regions/{}/subnetworks",
        client.compute_base_url(),
        region
    );
    let body = json!({
        "name": SUBNET_NAME,
        "network": vpc_url,
        "region": format!("https://www.googleapis.com/compute/v1/projects/{}/regions/{}", client.project_id, region),
        "ipCidrRange": subnet_cidr_for_region(region),
        "description": "byocvpn WireGuard subnet",
        "privateIpGoogleAccess": false,
        "stackType": "IPV4_IPV6",
        "ipv6AccessType": "EXTERNAL",
    });
    let operation = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: error.to_string(),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;

    let subnet = client.get(&url).await?;
    println!("GCP subnet '{}' created in {}.", SUBNET_NAME, region);
    Ok(subnet["selfLink"].as_str().unwrap_or_default().to_string())
}

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

/// Resolve the latest Ubuntu 22.04 LTS ARM64 image self-link for `VM_STANDARD.A1.Flex`-style
/// VMs. On GCP we use x86-64 (`e2-micro`), so we look up the standard amd64 family.
pub async fn get_ubuntu_image_self_link(client: &GcpClient) -> Result<String> {
    // The `ubuntu-os-cloud` project always has the latest Ubuntu 22.04 LTS image
    // under the `ubuntu-2204-lts` family.
    let url = "https://compute.googleapis.com/compute/v1/projects/ubuntu-os-cloud/global/images/family/ubuntu-2204-lts";
    let image =
        client
            .get(url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: format!("Failed to fetch Ubuntu 22.04 image: {}", error),
            })?;
    image["selfLink"]
        .as_str()
        .ok_or_else(|| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: "Ubuntu 22.04 image missing selfLink".to_string(),
            }
            .into()
        })
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Service Usage (API enablement)
// ---------------------------------------------------------------------------

const SERVICE_USAGE_BASE: &str = "https://serviceusage.googleapis.com/v1";
const COMPUTE_API_SERVICE: &str = "compute.googleapis.com";

/// Poll a Service Usage long-running operation until `done: true` (or times out).
async fn wait_for_service_usage_operation(client: &GcpClient, operation_name: &str) -> Result<()> {
    let url = format!("{}/{}", SERVICE_USAGE_BASE, operation_name);
    for attempt in 1..=30u32 {
        let operation = client.get(&url).await?;
        if operation["done"].as_bool() == Some(true) {
            if let Some(error) = operation.get("error") {
                let message = error["message"].as_str().unwrap_or("unknown error");
                return Err(ConfigurationError::InvalidCloudProvider(format!(
                    "Service Usage operation failed: {}",
                    message
                ))
                .into());
            }
            return Ok(());
        }
        eprintln!(
            "[GCP] Service Usage operation pending (attempt {}/30)...",
            attempt
        );
        sleep(Duration::from_secs(5)).await;
    }
    Err(ConfigurationError::InvalidCloudProvider(
        "Timed out waiting for Service Usage operation".to_string(),
    )
    .into())
}

/// Ensure the Compute Engine API is enabled for the project, enabling it if not.
/// Called automatically by `setup` and `enable_region` before any Compute API calls.
pub async fn ensure_compute_api_enabled(client: &GcpClient) -> Result<()> {
    let service_name = format!(
        "projects/{}/services/{}",
        client.project_id, COMPUTE_API_SERVICE
    );
    let url = format!("{}/{}", SERVICE_USAGE_BASE, service_name);

    let response = client.get(&url).await.map_err(|error| {
        ConfigurationError::InvalidCloudProvider(format!(
            "Failed to check Compute Engine API status: {}",
            error
        ))
    })?;

    if response["state"].as_str() == Some("ENABLED") {
        return Ok(());
    }

    eprintln!(
        "[GCP] Enabling Compute Engine API for project '{}'...",
        client.project_id
    );

    let enable_url = format!("{}:enable", url);
    let operation = client
        .post(&enable_url, &json!({}))
        .await
        .map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to enable Compute Engine API: {}",
                error
            ))
        })?;

    let operation_name = operation["name"].as_str().ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider(
            "Service Usage enable operation missing 'name' field".to_string(),
        )
    })?;

    wait_for_service_usage_operation(client, operation_name).await?;
    eprintln!(
        "[GCP] Compute Engine API enabled for project '{}'.",
        client.project_id
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Regions
// ---------------------------------------------------------------------------

/// Return all GCP regions the project can access, as `(region_name, continent)` pairs.
pub async fn list_regions(client: &GcpClient) -> Result<Vec<(String, String)>> {
    let url = format!("{}/regions", client.compute_base_url());
    let response =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?;

    let continent_prefixes: &[(&str, &str)] = &[
        ("us-", "United States"),
        ("northamerica-", "North America"),
        ("southamerica-", "South America"),
        ("europe-", "Europe"),
        ("asia-", "Asia Pacific"),
        ("australia-", "Asia Pacific"),
        ("me-", "Middle East"),
        ("africa-", "Africa"),
    ];

    let regions = response["items"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region| {
            let name = region["name"].as_str()?.to_string();
            // Only include regions that are UP.
            if region["status"].as_str() != Some("UP") {
                return None;
            }
            let continent = continent_prefixes
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix))
                .map(|(_, continent)| continent.to_string())
                .unwrap_or_else(|| "Other".to_string());
            Some((name, continent))
        })
        .collect();

    Ok(regions)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// First zone in a region (e.g. `us-central1` → `us-central1-a`).
/// GCP consistently provides `-a`, `-b`, `-c` zones for all regions.
pub fn primary_zone_for_region(region: &str) -> String {
    format!("{}-a", region)
}
