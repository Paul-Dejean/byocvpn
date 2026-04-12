use aws_config::{SdkConfig, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_ec2::{
    config::{Region, SharedCredentialsProvider},
    error::ProvideErrorMetadata,
};
use aws_sdk_ssm::{Client as SsmClient, error::SdkError};
use byocvpn_core::error::{ComputeProvisioningError, Result};

use crate::{aws_error::map_aws_error, provider::AwsProviderConfig};

const DEFAULT_REGION: &str = "us-east-1";
const CREDENTIALS_SOURCE: &str = "manual";
const AL2023_AMI_SSM_PARAMETER: &str =
    "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";
const AL2023_AMI_NAME: &str = "al2023";

pub(super) async fn get_sdk_config(
    config: &AwsProviderConfig,
    region: Option<String>,
) -> SdkConfig {
    let region_provider = match &region {
        Some(r) => RegionProviderChain::first_try(Region::new(r.clone())).or_default_provider(),
        None => RegionProviderChain::default_provider()
            .or_else(Region::new(DEFAULT_REGION)),
    };

    let mut config_loader = aws_config::from_env().region(region_provider);

    if let (Some(id), Some(secret)) = (&config.access_key_id, &config.secret_access_key) {
        let credentials = Credentials::new(id.clone(), secret.clone(), None, None, CREDENTIALS_SOURCE);
        let provider = SharedCredentialsProvider::new(credentials);
        config_loader = config_loader.credentials_provider(provider);
    }

    let config = config_loader.load().await;
    config
}

pub(super) async fn get_al2023_ami(ssm_client: &SsmClient) -> Result<String> {
    let result = ssm_client
        .get_parameter()
        .name(AL2023_AMI_SSM_PARAMETER)
        .send()
        .await
        .map_err(|sdk_error| match sdk_error {
            SdkError::ServiceError(service_error)
                if matches!(service_error.err().code(), Some("ParameterNotFound")) =>
            {
                ComputeProvisioningError::AmiLookupFailed {
                    name: AL2023_AMI_NAME.to_string(),
                    reason: "parameter not found".to_string(),
                }
                .into()
            }
            other => map_aws_error("get_parameter", other),
        })?;

    let ami_id = result
        .parameter()
        .and_then(|p| p.value())
        .ok_or(ComputeProvisioningError::AmiLookupFailed {
            name: AL2023_AMI_NAME.to_string(),
            reason: "parameter not found".to_string(),
        })?
        .to_string();

    Ok(ami_id)
}
