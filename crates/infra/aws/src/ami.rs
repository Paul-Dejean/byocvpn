use aws_sdk_ec2::error::ProvideErrorMetadata;
use aws_sdk_ssm::{Client as SsmClient, error::SdkError};
use byocvpn_core::error::{ComputeProvisioningError, Result};

use crate::aws_error::map_aws_error;

pub(super) const AL2023_AMI_SSM_PARAMETER: &str =
    "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";
const AL2023_AMI_NAME: &str = "al2023";

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
