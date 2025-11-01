use crate::cloud_provider::CloudProvider;

pub async fn setup(provider: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    provider.setup().await?;
    Ok(())
}

pub async fn enable_region(
    provider: &dyn CloudProvider,
    region: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    provider.enable_region(region).await?;
    Ok(())
}

#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub country: String,
}

pub async fn get_regions(
    provider: &dyn CloudProvider,
) -> Result<Vec<Region>, Box<dyn std::error::Error>> {
    // This is a placeholder implementation.
    // Replace with actual logic to fetch regions from the cloud provider.
    provider.get_regions().await
    //
}
