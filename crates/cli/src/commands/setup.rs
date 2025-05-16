use byocvpn_core::cloud_provider::CloudProvider;

pub async fn setup(aws: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    // aws.setup().await?;
    aws.enable_region("").await?;

    Ok(())
}
