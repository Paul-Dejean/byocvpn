use async_trait::async_trait;

use crate::{
    cloud_provider::{CloudProvider, CloudProviderName},
    error::{CredentialsError, Result},
};

#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn list_configured(&self) -> Result<Vec<CloudProviderName>>;
    async fn create(&self, provider: CloudProviderName) -> Result<Box<dyn CloudProvider>>;

    async fn require_configured(&self) -> Result<Vec<CloudProviderName>> {
        let configured_providers = self.list_configured().await?;
        if configured_providers.is_empty() {
            return Err(CredentialsError::NoProviderConfigured.into());
        }
        Ok(configured_providers)
    }
}
