use async_trait::async_trait;

use crate::{
    cloud_provider::{CloudProvider, CloudProviderName},
    error::Result,
};

#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn list_configured(&self) -> Result<Vec<CloudProviderName>>;
    async fn create(&self, provider: CloudProviderName) -> Result<Box<dyn CloudProvider>>;
}
