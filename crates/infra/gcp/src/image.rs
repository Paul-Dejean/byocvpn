use crate::{
    client::GcpClient,
    error::{GcpError, LabelOperation, Result},
};

const UBUNTU_IMAGE_PROJECT: &str = "ubuntu-os-cloud";
const UBUNTU_IMAGE_FAMILY: &str = "ubuntu-2204-lts";
const UBUNTU_IMAGE_NAME: &str = "Ubuntu 22.04 LTS";

pub async fn resolve_boot_image(client: &GcpClient) -> Result<String> {
    let lookup = client
        .images
        .get_from_family()
        .set_project(UBUNTU_IMAGE_PROJECT)
        .set_family(UBUNTU_IMAGE_FAMILY)
        .send()
        .await
        .label_operation("compute.images.get");
    let image = match lookup {
        Ok(image) => image,
        Err(error) if error.is_not_found() => {
            return Err(GcpError::BaseImageNotFound {
                image: UBUNTU_IMAGE_NAME,
            });
        }
        Err(error) => return Err(error),
    };
    image.self_link.ok_or(GcpError::MissingResourceField {
        resource: "image",
        field: "selfLink",
    })
}
