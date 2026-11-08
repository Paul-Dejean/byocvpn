use std::collections::HashSet;

use google_cloud_compute_v1::model::{
    Subnetwork,
    subnetwork::{Ipv6AccessType, StackType},
};
use log::info;

use crate::{
    client::GcpClient,
    error::{GcpError, LabelOperation, Result},
    network::vpc::build_vpc_link,
    operations::await_compute_operation,
};

const SUBNET_NAME: &str = "byocvpn-subnet";
const SUBNET_DESCRIPTION: &str = "byocvpn WireGuard subnet";

pub async fn ensure_subnet(client: &GcpClient, region: &str) -> Result<String> {
    match get_subnet(client, region).await? {
        Some(existing) => {
            let self_link = extract_self_link(&existing)?;
            if existing.stack_type != Some(StackType::Ipv4Ipv6) {
                upgrade_subnet_to_dual_stack(client, region, &existing).await?;
            }
            Ok(self_link)
        }
        None => create_subnet(client, region).await,
    }
}

async fn get_subnet(client: &GcpClient, region: &str) -> Result<Option<Subnetwork>> {
    let lookup = client
        .subnetworks
        .get()
        .set_project(&client.project_id)
        .set_region(region)
        .set_subnetwork(SUBNET_NAME)
        .send()
        .await
        .label_operation("compute.subnetworks.get");
    match lookup {
        Ok(subnet) => Ok(Some(subnet)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

async fn upgrade_subnet_to_dual_stack(
    client: &GcpClient,
    region: &str,
    existing: &Subnetwork,
) -> Result<()> {
    info!("[GCP] Upgrading subnet '{SUBNET_NAME}' in {region} to dual stack...");
    let patch_body = Subnetwork::new()
        .set_stack_type(StackType::Ipv4Ipv6)
        .set_ipv_6_access_type(Ipv6AccessType::External)
        .set_or_clear_fingerprint(existing.fingerprint.clone());
    let patch = client
        .subnetworks
        .patch()
        .set_project(&client.project_id)
        .set_region(region)
        .set_subnetwork(SUBNET_NAME)
        .set_body(patch_body)
        .poller();
    await_compute_operation("compute.subnetworks.update", patch).await?;
    info!("[GCP] Subnet '{SUBNET_NAME}' upgraded to dual stack in {region}.");
    Ok(())
}

async fn create_subnet(client: &GcpClient, region: &str) -> Result<String> {
    let subnetwork = Subnetwork::new()
        .set_name(SUBNET_NAME)
        .set_network(build_vpc_link(client))
        .set_region(client.build_resource_link(&format!("regions/{region}")))
        .set_ip_cidr_range(find_available_subnet_cidr(client).await?)
        .set_description(SUBNET_DESCRIPTION)
        .set_private_ip_google_access(false)
        .set_stack_type(StackType::Ipv4Ipv6)
        .set_ipv_6_access_type(Ipv6AccessType::External);
    let insert = client
        .subnetworks
        .insert()
        .set_project(&client.project_id)
        .set_region(region)
        .set_body(subnetwork)
        .poller();
    await_compute_operation("compute.subnetworks.create", insert).await?;

    let subnet = client
        .subnetworks
        .get()
        .set_project(&client.project_id)
        .set_region(region)
        .set_subnetwork(SUBNET_NAME)
        .send()
        .await
        .label_operation("compute.subnetworks.get")?;
    info!("[GCP] Subnet '{SUBNET_NAME}' created in {region}.");
    extract_self_link(&subnet)
}

async fn find_available_subnet_cidr(client: &GcpClient) -> Result<String> {
    let response = client
        .subnetworks
        .aggregated_list()
        .set_project(&client.project_id)
        .send()
        .await
        .label_operation("compute.subnetworks.list")?;
    let used_cidrs: HashSet<String> = response
        .items
        .into_values()
        .flat_map(|region_subnets| region_subnets.subnetworks)
        .filter_map(|subnet| subnet.ip_cidr_range)
        .collect();

    (0u16..=255)
        .flat_map(|second_octet| (0u16..16).map(move |block| (second_octet, block * 16)))
        .map(|(second_octet, third_octet)| format!("10.{second_octet}.{third_octet}.0/20"))
        .find(|candidate| !used_cidrs.contains(candidate))
        .ok_or(GcpError::NoSubnetCidrAvailable)
}

fn extract_self_link(subnet: &Subnetwork) -> Result<String> {
    subnet
        .self_link
        .clone()
        .ok_or(GcpError::MissingResourceField {
            resource: "subnet",
            field: "selfLink",
        })
}
