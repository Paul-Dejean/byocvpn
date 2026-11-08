use byocvpn_core::error::{NetworkProvisioningError, Result};
use log::info;

use crate::{
    client::GcpClient,
    models::{CreateVpcRequest, Operation, VpcResponse},
    operations::wait_for_compute_operation,
};

pub const VPC_NAME: &str = "byocvpn-vpc";
const VPC_DESCRIPTION: &str = "byocvpn WireGuard VPN network";

pub async fn ensure_vpc(client: &GcpClient) -> Result<String> {
    if let Some(self_link) = get_vpc(client).await? {
        return Ok(self_link);
    }
    create_vpc(client).await
}

pub fn build_vpc_link(client: &GcpClient) -> String {
    client.build_resource_link(&format!("global/networks/{VPC_NAME}"))
}

async fn get_vpc(client: &GcpClient) -> Result<Option<String>> {
    let url = client.build_compute_url(&format!("global/networks/{VPC_NAME}"));
    match client.get::<VpcResponse>(&url).await {
        Ok(vpc) => Ok(Some(extract_self_link(vpc)?)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

async fn create_vpc(client: &GcpClient) -> Result<String> {
    let body = CreateVpcRequest {
        name: VPC_NAME.to_string(),
        auto_create_subnetworks: false,
        description: VPC_DESCRIPTION.to_string(),
    };
    let operation: Operation = client
        .post(&client.build_compute_url("global/networks"), &body)
        .await
        .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        })?;
    wait_for_compute_operation(client, &operation).await?;

    let vpc: VpcResponse = client
        .get(&client.build_compute_url(&format!("global/networks/{VPC_NAME}")))
        .await?;
    info!("[GCP] VPC '{VPC_NAME}' created.");
    extract_self_link(vpc)
}

fn extract_self_link(vpc: VpcResponse) -> Result<String> {
    vpc.self_link.ok_or_else(|| {
        NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "VPC",
        }
        .into()
    })
}
