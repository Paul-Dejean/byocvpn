use crate::{
    cloud_provider::{CloudProvider, InstanceInfo},
    error::Result,
};

pub async fn list_instances(provider: &dyn CloudProvider) -> Result<Vec<InstanceInfo>> {
    provider.list_instances().await
}
