use std::collections::HashSet;

use byocvpn_core::error::{NetworkProvisioningError, Result};
use log::info;

use crate::{
    client::GcpClient,
    models::{
        AggregatedSubnetListResponse, CreateSubnetRequest, Operation, PatchSubnetRequest,
        SubnetResponse,
    },
    network::vpc::build_vpc_link,
    operations::wait_for_compute_operation,
};

const SUBNET_NAME: &str = "byocvpn-subnet";
const SUBNET_DESCRIPTION: &str = "byocvpn WireGuard subnet";
const DUAL_STACK_TYPE: &str = "IPV4_IPV6";
const EXTERNAL_IPV6_ACCESS_TYPE: &str = "EXTERNAL";

pub async fn ensure_subnet(client: &GcpClient, region: &str) -> Result<String> {
    match get_subnet(client, region).await? {
        Some(existing) => {
            let self_link = extract_self_link(&existing)?;
            if existing.stack_type.as_deref() != Some(DUAL_STACK_TYPE) {
                upgrade_subnet_to_dual_stack(client, region, &existing).await?;
            }
            Ok(self_link)
        }
        None => create_subnet(client, region).await,
    }
}

async fn get_subnet(client: &GcpClient, region: &str) -> Result<Option<SubnetResponse>> {
    let url = build_subnet_url(client, region);
    match client.get::<SubnetResponse>(&url).await {
        Ok(subnet) => Ok(Some(subnet)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

async fn upgrade_subnet_to_dual_stack(
    client: &GcpClient,
    region: &str,
    existing: &SubnetResponse,
) -> Result<()> {
    info!("[GCP] Upgrading subnet '{SUBNET_NAME}' in {region} to {DUAL_STACK_TYPE}...");
    let body = PatchSubnetRequest {
        stack_type: DUAL_STACK_TYPE.to_string(),
        ipv6_access_type: EXTERNAL_IPV6_ACCESS_TYPE.to_string(),
        fingerprint: existing.fingerprint.clone().unwrap_or_default(),
    };
    let operation: Operation = client
        .patch(&build_subnet_url(client, region), &body)
        .await
        .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
            reason: format!("Failed to upgrade subnet to {DUAL_STACK_TYPE}: {error}"),
        })?;
    wait_for_compute_operation(client, &operation)
        .await
        .map_err(|error| error.with_context("subnet dual-stack upgrade operation failed"))?;
    info!("[GCP] Subnet '{SUBNET_NAME}' upgraded to {DUAL_STACK_TYPE} in {region}.");
    Ok(())
}

async fn create_subnet(client: &GcpClient, region: &str) -> Result<String> {
    let body = CreateSubnetRequest {
        name: SUBNET_NAME.to_string(),
        network: build_vpc_link(client),
        region: client.build_resource_link(&format!("regions/{region}")),
        ip_cidr_range: find_available_subnet_cidr(client).await?,
        description: SUBNET_DESCRIPTION.to_string(),
        private_ip_google_access: false,
        stack_type: DUAL_STACK_TYPE.to_string(),
        ipv6_access_type: EXTERNAL_IPV6_ACCESS_TYPE.to_string(),
    };
    let operation: Operation = client
        .post(
            &client.build_compute_url(&format!("regions/{region}/subnetworks")),
            &body,
        )
        .await
        .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
            reason: error.to_string(),
        })?;
    wait_for_compute_operation(client, &operation).await?;

    let subnet: SubnetResponse = client.get(&build_subnet_url(client, region)).await?;
    info!("[GCP] Subnet '{SUBNET_NAME}' created in {region}.");
    extract_self_link(&subnet)
}

async fn find_available_subnet_cidr(client: &GcpClient) -> Result<String> {
    let response: AggregatedSubnetListResponse = client
        .get(&client.build_compute_url("aggregated/subnetworks"))
        .await?;
    let used_cidrs: HashSet<String> = response
        .items
        .unwrap_or_default()
        .into_values()
        .flat_map(|region_subnets| region_subnets.subnetworks.unwrap_or_default())
        .filter_map(|subnet| subnet.ip_cidr_range)
        .collect();

    (0u16..=255)
        .flat_map(|second_octet| (0u16..16).map(move |block| (second_octet, block * 16)))
        .map(|(second_octet, third_octet)| format!("10.{second_octet}.{third_octet}.0/20"))
        .find(|candidate| !used_cidrs.contains(candidate))
        .ok_or_else(|| {
            NetworkProvisioningError::SubnetCreationFailed {
                reason: "no available /20 CIDR block in 10.0.0.0/8".to_string(),
            }
            .into()
        })
}

fn build_subnet_url(client: &GcpClient, region: &str) -> String {
    client.build_compute_url(&format!("regions/{region}/subnetworks/{SUBNET_NAME}"))
}

fn extract_self_link(subnet: &SubnetResponse) -> Result<String> {
    subnet.self_link.clone().ok_or_else(|| {
        NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "subnet",
        }
        .into()
    })
}
