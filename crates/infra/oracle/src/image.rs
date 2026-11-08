use byocvpn_core::error::{NetworkProvisioningError, Result};

use crate::{client::OciClient, models::ImageResponse};

const IMAGE_OPERATING_SYSTEM: &str = "Canonical+Ubuntu";
const IMAGE_OPERATING_SYSTEM_VERSION: &str = "22.04";
const IMAGE_SHAPE: &str = "VM.Standard.A1.Flex";
const IMAGE_NAME: &str = "Ubuntu 22.04 ARM";

pub async fn resolve_boot_image(client: &OciClient) -> Result<String> {
    let url = client.build_core_url(&format!(
        "/images?compartmentId={}&operatingSystem={IMAGE_OPERATING_SYSTEM}&operatingSystemVersion={IMAGE_OPERATING_SYSTEM_VERSION}&shape={IMAGE_SHAPE}&lifecycleState=AVAILABLE",
        client.get_compartment_ocid()
    ));
    let images: Vec<ImageResponse> = client.get(&url).await?;
    images
        .into_iter()
        .next()
        .map(|image| image.id)
        .ok_or_else(|| {
            NetworkProvisioningError::BaseImageNotFound {
                image: IMAGE_NAME.to_string(),
            }
            .into()
        })
}
