use log::{debug, info};
use serde::de::IgnoredAny;

use crate::{
    arm_resource::ArmResource,
    client::AzureClient,
    error::Result,
    models::{ResourceGroupRequest, build_managed_tags},
    resource_operations::{get_existing, put_and_wait},
};

pub async fn ensure_resource_group(client: &AzureClient, location: &str) -> Result<()> {
    let resource = ArmResource::resource_group(location);
    if get_existing::<IgnoredAny>(client, &resource)
        .await?
        .is_some()
    {
        debug!("[Azure] {resource} already exists, skipping creation.");
        return Ok(());
    }

    info!("[Azure] Creating {resource}...");
    let body = ResourceGroupRequest {
        location: location.to_string(),
        tags: build_managed_tags(),
    };
    put_and_wait(client, &resource, &body).await?;
    info!("[Azure] {resource} created.");
    Ok(())
}
