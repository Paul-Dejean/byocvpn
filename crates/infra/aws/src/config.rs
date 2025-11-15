use aws_config::{SdkConfig, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_ec2::{
    config::{Region, SharedCredentialsProvider},
    error::ProvideErrorMetadata,
};
use aws_sdk_ssm::{Client as SsmClient, error::SdkError};
use byocvpn_core::error::{Error, Result};

use crate::{provider::AwsProviderConfig, ssm_error::map_aws_ssm_error};

pub(super) async fn get_config(config: &AwsProviderConfig) -> SdkConfig {
    let region_provider = match &config.region {
        Some(r) => RegionProviderChain::first_try(Region::new(r.clone())).or_default_provider(),
        None => RegionProviderChain::default_provider(),
    };

    // Begin building config
    let mut config_loader = aws_config::from_env().region(region_provider);

    // Optionally override credentials
    if let (Some(id), Some(secret)) = (&config.access_key_id, &config.secret_access_key) {
        let credentials = Credentials::new(id.clone(), secret.clone(), None, None, "manual");
        let provider = SharedCredentialsProvider::new(credentials);
        config_loader = config_loader.credentials_provider(provider);
    }

    let config = config_loader.load().await;
    config
}

pub(super) async fn get_al2023_ami(ssm_client: &SsmClient) -> Result<String> {
    // AL2023 x86_64 SSM parameter name
    let param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";

    // Fetch the parameter value (AMI ID)
    let result = ssm_client
        .get_parameter()
        .name(param_name)
        .send()
        .await
        .map_err(|sdk_error| match sdk_error {
            SdkError::ServiceError(service_error)
                if matches!(service_error.err().code(), Some("ParameterNotFound")) =>
            {
                Error::BaseImageNotFound("al2023".to_string())
            }
            other => map_aws_ssm_error("get_parameter", other),
        })?;

    let ami_id = result
        .parameter()
        .and_then(|p| p.value())
        .ok_or(Error::BaseImageNotFound("al2023".to_string()))?
        .to_string();

    Ok(ami_id)
}
