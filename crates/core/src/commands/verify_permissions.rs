use serde_json::{Value, json};

use crate::{cloud_provider::CloudProvider, error::Result};
pub async fn verify_permissions(cloud_provider: &dyn CloudProvider) -> Result<Value> {
    cloud_provider.verify_permissions().await
}
