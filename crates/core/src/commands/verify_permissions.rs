use crate::{
    cloud_provider::{CloudProvider, PermissionStatus},
    error::Result,
};

pub async fn verify_permissions(
    cloud_provider: &dyn CloudProvider,
) -> Result<Vec<PermissionStatus>> {
    cloud_provider.verify_permissions().await
}
