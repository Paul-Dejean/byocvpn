use byocvpn_core::error::{Error, NetworkProvisioningError, Result};

use crate::{client::GcpClient, models::ImageResponse};

const UBUNTU_IMAGE_FAMILY_URL: &str = "https://compute.googleapis.com/compute/v1/projects/ubuntu-os-cloud/global/images/family/ubuntu-2204-lts";
const UBUNTU_IMAGE_NAME: &str = "Ubuntu 22.04 LTS";

pub async fn resolve_boot_image(client: &GcpClient) -> Result<String> {
    let image: ImageResponse = client
        .get(UBUNTU_IMAGE_FAMILY_URL)
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
