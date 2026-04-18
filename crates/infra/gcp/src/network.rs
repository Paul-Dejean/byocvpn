use byocvpn_core::error::{Error, NetworkProvisioningError, Result};
use byocvpn_core::retry::retry;
use tokio::time::Duration;

use crate::client::GcpClient;
use crate::models::{
    CreateFirewallRuleRequest, CreateSubnetRequest, CreateVpcRequest, EmptyRequest,
    FirewallAllowed, FirewallRuleResponse, ImageResponse, Operation, PatchFirewallRuleRequest,
    PatchSubnetRequest, RegionListResponse, ServiceResponse, SubnetResponse, VpcResponse,
};
use log::*;

const VPC_NAME: &str = "byocvpn-vpc";
const SUBNET_NAME: &str = "byocvpn-subnet";
const FIREWALL_NAME_IPV4: &str = "byocvpn-wireguard-ipv4";
const FIREWALL_NAME_IPV6: &str = "byocvpn-wireguard-ipv6";
const FIREWALL_TAG: &str = "byocvpn";
const IPV4_ALL_CIDR: &str = "0.0.0.0/0";
const IPV6_ALL_CIDR: &str = "::/0";

async fn wait_for_operation(client: &GcpClient, operation_url: &str) -> Result<()> {
    retry(
        || async move {
            let operation: Operation = client.get(operation_url).await?;
            match operation.status.as_deref() {
                Some("DONE") => {
                    if let Some(error) = operation.error {
                        let message = error
                            .errors
                            .as_ref()
                            .and_then(|errors| errors.first())
                            .and_then(|detail| detail.message.as_deref())
                            .unwrap_or("unknown error")
                            .to_string();
                        return Err(
                            NetworkProvisioningError::CloudOperationFailed { reason: message }.into(),
                        );
                    }
                    Ok(())
                }
                Some("PENDING") | Some("RUNNING") => {
                    debug!("[GCP] operation pending: {}", operation_url);
                    Err(NetworkProvisioningError::CloudOperationTimedOut {
                        operation: operation_url.to_string(),
                    }
                    .into())
                }
                other => Err(NetworkProvisioningError::CloudOperationFailed {
                    reason: format!("Unexpected GCP operation status: {:?}", other),
                }
                .into()),
            }
        },
        60,
        Duration::from_secs(3),
    )
    .await
}

async fn wait_for_operation_response(client: &GcpClient, operation: &Operation) -> Result<()> {
    let operation_url = operation
        .self_link
        .as_deref()
        .ok_or_else(|| NetworkProvisioningError::CloudOperationFailed {
            reason: "operation response missing selfLink".to_string(),
        })?
        .to_string();
    wait_for_operation(client, &operation_url).await
}

async fn get_vpc(client: &GcpClient) -> Result<Option<String>> {
    let url = format!("{}/global/networks/{}", client.build_compute_base_url(), VPC_NAME);
    match client.get::<VpcResponse>(&url).await {
        Ok(vpc) => {
            let self_link = vpc.self_link.ok_or(NetworkProvisioningError::MissingResourceField {
                field: "selfLink",
                resource: "VPC",
            })?;
            Ok(Some(self_link))
        }
        Err(Error::Network(NetworkProvisioningError::ResourceNotFound { .. })) => Ok(None),
        Err(error) => Err(error),
    }
}

async fn create_vpc(client: &GcpClient) -> Result<String> {
    let url = format!("{}/global/networks/{}", client.build_compute_base_url(), VPC_NAME);
    let create_url = format!("{}/global/networks", client.build_compute_base_url());
    let body = CreateVpcRequest {
        name: VPC_NAME.to_string(),
        auto_create_subnetworks: false,
        description: "byocvpn WireGuard VPN network".to_string(),
    };
    let operation: Operation = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;
    let vpc: VpcResponse = client.get(&url).await?;
    info!("GCP VPC '{}' created.", VPC_NAME);
    vpc.self_link.ok_or_else(|| {
        NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "VPC",
        }
        .into()
    })
}

pub async fn ensure_vpc(client: &GcpClient) -> Result<String> {
    if let Some(self_link) = get_vpc(client).await? {
        return Ok(self_link);
    }
    create_vpc(client).await
}

async fn get_firewall_rule(client: &GcpClient, name: &str) -> Option<FirewallRuleResponse> {
    let url = format!("{}/global/firewalls/{}", client.build_compute_base_url(), name);
    client.get::<FirewallRuleResponse>(&url).await.ok()
}

fn build_desired_firewall_allowed() -> Vec<FirewallAllowed> {
    vec![
        FirewallAllowed {
            ip_protocol: "udp".to_string(),
            ports: vec!["51820".to_string()],
        },
        FirewallAllowed {
            ip_protocol: "tcp".to_string(),
            ports: vec!["51820".to_string()],
        },
    ]
}

fn firewall_rule_matches_desired_state(existing_rule: &FirewallRuleResponse, source_range: &str) -> bool {
    let desired_allowed = build_desired_firewall_allowed();
    let desired_source_ranges = vec![source_range.to_string()];
    let desired_target_tags = vec![FIREWALL_TAG.to_string()];

    existing_rule.allowed.as_deref() == Some(&desired_allowed)
        && existing_rule.source_ranges.as_deref() == Some(&desired_source_ranges)
        && existing_rule.target_tags.as_deref() == Some(&desired_target_tags)
}

async fn patch_firewall_rule(client: &GcpClient, name: &str, source_range: &str) -> Result<()> {
    let url = format!("{}/global/firewalls/{}", client.build_compute_base_url(), name);
    let body = PatchFirewallRuleRequest {
        allowed: build_desired_firewall_allowed(),
        source_ranges: vec![source_range.to_string()],
        target_tags: vec![FIREWALL_TAG.to_string()],
        direction: "INGRESS".to_string(),
        priority: 1000,
    };
    let operation: Operation = client.patch(&url, &body).await.map_err(|error| {
        NetworkProvisioningError::SecurityGroupCreationFailed {
            reason: format!("Failed to patch firewall rule '{}': {}", name, error),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;
    info!("GCP firewall rule '{}' patched.", name);
    Ok(())
}

async fn create_firewall_rule(
    client: &GcpClient,
    name: &str,
    vpc_url: &str,
    description: &str,
    source_range: &str,
) -> Result<()> {
    let create_url = format!("{}/global/firewalls", client.build_compute_base_url());
    let body = CreateFirewallRuleRequest {
        name: name.to_string(),
        network: vpc_url.to_string(),
        description: description.to_string(),
        direction: "INGRESS".to_string(),
        priority: 1000,
        target_tags: vec![FIREWALL_TAG.to_string()],
        allowed: build_desired_firewall_allowed(),
        source_ranges: vec![source_range.to_string()],
    };
    let operation: Operation = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::SecurityGroupCreationFailed {
            reason: error.to_string(),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;
    info!("GCP firewall rule '{}' created.", name);
    Ok(())
}

pub async fn ensure_firewall_rules(client: &GcpClient) -> Result<()> {
    let vpc_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/global/networks/{}",
        client.project_id, VPC_NAME
    );

    for (rule_name, source_range, description) in [
        (FIREWALL_NAME_IPV4, IPV4_ALL_CIDR, "Allow WireGuard UDP and health TCP on 51820 (IPv4) for byocvpn"),
        (FIREWALL_NAME_IPV6, IPV6_ALL_CIDR, "Allow WireGuard UDP and health TCP on 51820 (IPv6) for byocvpn"),
    ] {
        match get_firewall_rule(client, rule_name).await {
            Some(existing_rule) => {
                if !firewall_rule_matches_desired_state(&existing_rule, source_range) {
                    info!("[GCP] Firewall rule '{}' has drifted, patching...", rule_name);
                    patch_firewall_rule(client, rule_name, source_range).await?;
                }
            }
            None => {
                create_firewall_rule(client, rule_name, &vpc_url, description, source_range).await?;
            }
        }
    }

    Ok(())
}

fn compute_subnet_cidr_for_region(region: &str) -> String {
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

    let hash: u8 = region
        .bytes()
        .enumerate()
        .fold(0u32, |acc, (index, byte)| {
            acc.wrapping_add((byte as u32).wrapping_mul((index + 1) as u32))
        })
        .wrapping_rem(40) as u8;
    format!("10.{}.0.0/20", 200 + hash)
}

async fn get_subnet(client: &GcpClient, region: &str) -> Result<Option<SubnetResponse>> {
    let url = format!("{}/regions/{}/subnetworks/{}", client.build_compute_base_url(), region, SUBNET_NAME);
    match client.get::<SubnetResponse>(&url).await {
        Ok(subnet) => Ok(Some(subnet)),
        Err(Error::Network(NetworkProvisioningError::ResourceNotFound { .. })) => Ok(None),
        Err(error) => Err(error),
    }
}

async fn create_subnet(client: &GcpClient, region: &str) -> Result<String> {
    let url = format!("{}/regions/{}/subnetworks/{}", client.build_compute_base_url(), region, SUBNET_NAME);
    let vpc_url = format!(
        "https://www.googleapis.com/compute/v1/projects/{}/global/networks/{}",
        client.project_id, VPC_NAME
    );
    let create_url = format!("{}/regions/{}/subnetworks", client.build_compute_base_url(), region);
    let body = CreateSubnetRequest {
        name: SUBNET_NAME.to_string(),
        network: vpc_url,
        region: format!(
            "https://www.googleapis.com/compute/v1/projects/{}/regions/{}",
            client.project_id, region
        ),
        ip_cidr_range: compute_subnet_cidr_for_region(region),
        description: "byocvpn WireGuard subnet".to_string(),
        private_ip_google_access: false,
        stack_type: "IPV4_IPV6".to_string(),
        ipv6_access_type: "EXTERNAL".to_string(),
    };
    let operation: Operation = client.post(&create_url, &body).await.map_err(|error| {
        NetworkProvisioningError::SubnetCreationFailed {
            reason: error.to_string(),
        }
    })?;
    wait_for_operation_response(client, &operation).await?;
    let subnet: SubnetResponse = client.get(&url).await?;
    info!("GCP subnet '{}' created in {}.", SUBNET_NAME, region);
    subnet.self_link.ok_or_else(|| {
        NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "subnet",
        }
        .into()
    })
}

pub async fn ensure_subnet(client: &GcpClient, region: &str) -> Result<String> {
    let url = format!("{}/regions/{}/subnetworks/{}", client.build_compute_base_url(), region, SUBNET_NAME);
    if let Some(existing) = get_subnet(client, region).await? {
        let self_link = existing.self_link.clone().ok_or(NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "subnet",
        })?;
        if existing.stack_type.as_deref() != Some("IPV4_IPV6") {
            info!("[GCP] Upgrading subnet '{}' in {} to IPV4_IPV6...", SUBNET_NAME, region);
            let fingerprint = existing.fingerprint.clone().unwrap_or_default();
            let patch_body = PatchSubnetRequest {
                stack_type: "IPV4_IPV6".to_string(),
                ipv6_access_type: "EXTERNAL".to_string(),
                fingerprint,
            };
            let operation: Operation = client
                .patch(&url, &patch_body)
                .await
                .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                    reason: format!("Failed to upgrade subnet to IPV4_IPV6: {}", error),
                })?;
            wait_for_operation_response(client, &operation)
                .await
                .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                    reason: format!("Subnet IPV4_IPV6 upgrade operation failed: {}", error),
                })?;
            info!("GCP subnet '{}' upgraded to IPV4_IPV6 in {}.", SUBNET_NAME, region);
        }
        return Ok(self_link);
    }
    create_subnet(client, region).await
}

pub async fn get_ubuntu_image_self_link(client: &GcpClient) -> Result<String> {
    let url = "https://compute.googleapis.com/compute/v1/projects/ubuntu-os-cloud/global/images/family/ubuntu-2204-lts";
    let image: ImageResponse = client
        .get(url)
        .await
        .map_err(|_| NetworkProvisioningError::BaseImageNotFound {
            image: "Ubuntu 22.04 LTS".to_string(),
        })?;
    image.self_link.ok_or_else(|| {
        NetworkProvisioningError::BaseImageNotFound {
            image: "Ubuntu 22.04 LTS".to_string(),
        }
        .into()
    })
}

const SERVICE_USAGE_BASE: &str = "https://serviceusage.googleapis.com/v1";
const COMPUTE_API_SERVICE: &str = "compute.googleapis.com";

async fn wait_for_service_usage_operation(client: &GcpClient, operation_name: &str) -> Result<()> {
    let url = format!("{}/{}", SERVICE_USAGE_BASE, operation_name);
    retry(
        || {
            let url = url.clone();
            async move {
                let operation: Operation = client.get(&url).await?;
                if operation.done == Some(true) {
                    if let Some(error) = operation.error {
                        let message = error.message.unwrap_or_else(|| "unknown error".to_string());
                        return Err(
                            NetworkProvisioningError::CloudOperationFailed { reason: message }.into(),
                        );
                    }
                    return Ok(());
                }
                debug!("[GCP] Service Usage operation pending...");
                Err(NetworkProvisioningError::CloudOperationTimedOut {
                    operation: "Service Usage".to_string(),
                }
                .into())
            }
        },
        30,
        Duration::from_secs(5),
    )
    .await
}

pub async fn ensure_compute_api_enabled(client: &GcpClient) -> Result<()> {
    let service_name = format!(
        "projects/{}/services/{}",
        client.project_id, COMPUTE_API_SERVICE
    );
    let url = format!("{}/{}", SERVICE_USAGE_BASE, service_name);

    let response: ServiceResponse =
        client
            .get(&url)
            .await
            .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
                step: "Compute Engine API check".to_string(),
                reason: error.to_string(),
            })?;

    if response.state.as_deref() == Some("ENABLED") {
        return Ok(());
    }

    info!(
        "[GCP] Enabling Compute Engine API for project '{}'...",
        client.project_id
    );

    let enable_url = format!("{}:enable", url);
    let operation: Operation = client
        .post(&enable_url, &EmptyRequest {})
        .await
        .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
            step: "Compute Engine API enablement".to_string(),
            reason: error.to_string(),
        })?;

    let operation_name = operation.name.as_deref().ok_or_else(|| {
        NetworkProvisioningError::ProviderSetupFailed {
            step: "Compute Engine API enablement".to_string(),
            reason: "operation response missing 'name' field".to_string(),
        }
    })?;

    wait_for_service_usage_operation(client, operation_name).await?;
    info!(
        "[GCP] Compute Engine API enabled for project '{}'.",
        client.project_id
    );
    Ok(())
}

pub async fn list_regions(client: &GcpClient) -> Result<Vec<(String, String)>> {
    let url = format!("{}/regions", client.build_compute_base_url());
    let response: RegionListResponse =
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

    let regions = response
        .items
        .unwrap_or_default()
        .into_iter()
        .filter_map(|region_item| {
            let name = region_item.name?;

            if region_item.status.as_deref() != Some("UP") {
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

pub fn build_primary_zone_for_region(region: &str) -> String {
    format!("{}-a", region)
}
