use byocvpn_core::managed_resources::build_managed_tags;
use serde::de::IgnoredAny;
use tracing::{debug, info};

use crate::{
    arm_resource::ArmResource,
    client::AzureClient,
    error::Result,
    models::ResourceGroupRequest,
    resource_operations::{get_existing, put_and_wait},
};

pub async fn ensure_resource_group(client: &AzureClient, location: &str) -> Result<()> {
    let resource = ArmResource::resource_group(location);
    if get_existing::<IgnoredAny>(client, &resource)
        .await?
        .is_some()
    {
        debug!(%resource, "already exists");
        return Ok(());
    }

    debug!(%resource, "creating");
    let body = ResourceGroupRequest {
        location: location.to_string(),
        tags: build_managed_tags(),
    };
    put_and_wait(client, &resource, &body).await?;
    info!(%resource, "created");
    Ok(())
}
