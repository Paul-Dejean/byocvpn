use byocvpn_core::cloud_provider::CloudProvider;

pub async fn setup(aws: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    aws.enable_region("").await?;
    Ok(())
}
