use crate::{cloud_provider::CloudProvider, error::Result};

pub async fn setup(provider: &dyn CloudProvider) -> Result<()> {
    provider.setup().await?;
    Ok(())
}

pub async fn enable_region(provider: &dyn CloudProvider, region: &str) -> Result<()> {
    provider.enable_region(region).await?;
    Ok(())
}

#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub country: String,
}

pub async fn get_regions(provider: &dyn CloudProvider) -> Result<Vec<Region>> {
    provider.get_regions().await
}
