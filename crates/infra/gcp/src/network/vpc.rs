use byocvpn_core::error::{NetworkProvisioningError, Result};
use google_cloud_compute_v1::model::Network;
use log::info;

use crate::{client::GcpClient, error::convert_sdk_error, operations::await_compute_operation};

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
    let lookup = client
        .networks
        .get()
        .set_project(&client.project_id)
        .set_network(VPC_NAME)
        .send()
        .await
        .map_err(|error| convert_sdk_error("getting VPC", error));
    match lookup {
        Ok(vpc) => Ok(Some(extract_self_link(vpc)?)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

async fn create_vpc(client: &GcpClient) -> Result<String> {
    let network = Network::new()
        .set_name(VPC_NAME)
        .set_auto_create_subnetworks(false)
        .set_description(VPC_DESCRIPTION);
    let insert = client
        .networks
        .insert()
        .set_project(&client.project_id)
        .set_body(network)
        .poller();
    await_compute_operation("creating VPC", insert)
        .await
        .map_err(|error| NetworkProvisioningError::VpcCreationFailed {
            reason: error.to_string(),
        })?;

    let vpc = client
        .networks
        .get()
        .set_project(&client.project_id)
        .set_network(VPC_NAME)
        .send()
        .await
        .map_err(|error| convert_sdk_error("getting VPC", error))?;
    info!("[GCP] VPC '{VPC_NAME}' created.");
    extract_self_link(vpc)
}

fn extract_self_link(vpc: Network) -> Result<String> {
    vpc.self_link.ok_or_else(|| {
        NetworkProvisioningError::MissingResourceField {
            field: "selfLink",
            resource: "VPC",
        }
        .into()
    })
}
