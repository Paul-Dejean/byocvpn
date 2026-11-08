use crate::models::ImageReference;

const IMAGE_PUBLISHER: &str = "Canonical";
const IMAGE_OFFER: &str = "0001-com-ubuntu-server-jammy";
const IMAGE_SKU: &str = "22_04-lts-gen2";
const IMAGE_VERSION: &str = "latest";

pub fn build_image_reference() -> ImageReference {
    ImageReference {
        publisher: IMAGE_PUBLISHER.to_string(),
        offer: IMAGE_OFFER.to_string(),
        sku: IMAGE_SKU.to_string(),
        version: IMAGE_VERSION.to_string(),
    }
}
