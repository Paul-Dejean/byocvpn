use aws_config::{SdkConfig, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_ec2::config::{Region, SharedCredentialsProvider};

use crate::provider::AwsProviderConfig;

const DEFAULT_REGION: &str = "us-east-1";

pub(super) async fn get_sdk_config(
    config: &AwsProviderConfig,
    region: Option<String>,
) -> SdkConfig {
    let region_provider = match &region {
        Some(r) => RegionProviderChain::first_try(Region::new(r.clone())).or_default_provider(),
        None => RegionProviderChain::default_provider().or_else(Region::new(DEFAULT_REGION)),
    };

    let mut config_loader = aws_config::from_env().region(region_provider);

    if let (Some(access_key_id), Some(secret_access_key)) =
        (&config.access_key_id, &config.secret_access_key)
    {
        let credentials = Credentials::new(
            access_key_id.clone(),
            secret_access_key.clone(),
            None,
            None,
            "manual",
        );
        let provider = SharedCredentialsProvider::new(credentials);
        config_loader = config_loader.credentials_provider(provider);
    }

    let config = config_loader.load().await;
    config
}
