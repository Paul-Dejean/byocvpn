use byocvpn_core::error::{Error, NetworkProvisioningError, Result};

use crate::client::GcpClient;

const UBUNTU_IMAGE_PROJECT: &str = "ubuntu-os-cloud";
const UBUNTU_IMAGE_FAMILY: &str = "ubuntu-2204-lts";
const UBUNTU_IMAGE_NAME: &str = "Ubuntu 22.04 LTS";

pub async fn resolve_boot_image(client: &GcpClient) -> Result<String> {
    let image = client
        .images
        .get_from_family()
        .set_project(UBUNTU_IMAGE_PROJECT)
        .set_family(UBUNTU_IMAGE_FAMILY)
        .send()
        .await
        .map_err(|_| build_image_not_found_error())?;
    image.self_link.ok_or_else(build_image_not_found_error)
}

fn build_image_not_found_error() -> Error {
    NetworkProvisioningError::BaseImageNotFound {
        image: UBUNTU_IMAGE_NAME.to_string(),
    }
    .into()
}
