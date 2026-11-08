use aws_config::{SdkConfig, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_ec2::config::{Region, SharedCredentialsProvider};

use crate::credentials::AwsCredentials;

const DEFAULT_REGION: &str = "us-east-1";
const CREDENTIALS_PROVIDER_NAME: &str = "manual";

pub async fn load_sdk_config(credentials: &AwsCredentials, region: Option<&str>) -> SdkConfig {
    let region_provider = match region {
        Some(region_name) => RegionProviderChain::first_try(Region::new(region_name.to_string()))
            .or_default_provider(),
        None => RegionProviderChain::default_provider().or_else(Region::new(DEFAULT_REGION)),
    };
    let sdk_credentials = Credentials::new(
        credentials.access_key_id.clone(),
        credentials.secret_access_key.clone(),
        None,
        None,
        CREDENTIALS_PROVIDER_NAME,
    );
    aws_config::from_env()
        .region(region_provider)
        .credentials_provider(SharedCredentialsProvider::new(sdk_credentials))
        .load()
        .await
}
