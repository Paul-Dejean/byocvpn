use aws_config::{BehaviorVersion, SdkConfig};
use aws_credential_types::Credentials;
use aws_sdk_ec2::config::{Region, SharedCredentialsProvider};

use crate::credentials::AwsCredentials;

const DEFAULT_REGION: &str = "us-east-1";
const CREDENTIALS_PROVIDER_NAME: &str = "byocvpn";

pub fn build_sdk_config(credentials: &AwsCredentials, region: Option<&str>) -> SdkConfig {
    let sdk_credentials = Credentials::new(
        credentials.access_key_id.clone(),
        credentials.secret_access_key.clone(),
        None,
        None,
        CREDENTIALS_PROVIDER_NAME,
    );
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(region.unwrap_or(DEFAULT_REGION).to_string()))
        .credentials_provider(SharedCredentialsProvider::new(sdk_credentials))
        .build()
}
