use crate::cloud_provider::{CloudProvider, InstanceInfo};

pub async fn list_instances(
    provider: &dyn CloudProvider,
) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>> {
    provider.list_instances().await
}
