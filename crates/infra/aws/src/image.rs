use aws_sdk_ssm::error::{ProvideErrorMetadata, SdkError};
use byocvpn_core::error::{ComputeProvisioningError, Error, Result};

use crate::{client::AwsClient, error::map_aws_error};

const AL2023_AMI_SSM_PARAMETER: &str =
    "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";
const AL2023_AMI_NAME: &str = "al2023";
const GET_PARAMETER_OPERATION: &str = "ssm:GetParameter";

pub async fn resolve_boot_image(client: &AwsClient) -> Result<String> {
    let response = client
        .ssm
        .get_parameter()
        .name(AL2023_AMI_SSM_PARAMETER)
        .send()
        .await
        .map_err(|sdk_error| match sdk_error {
            SdkError::ServiceError(service_error)
                if matches!(service_error.err().code(), Some("ParameterNotFound")) =>
            {
                build_ami_lookup_error("parameter not found")
            }
            other => map_aws_error(GET_PARAMETER_OPERATION, other),
        })?;

    response
        .parameter()
        .and_then(|parameter| parameter.value())
        .map(ToString::to_string)
        .ok_or_else(|| build_ami_lookup_error("parameter has no value"))
}

fn build_ami_lookup_error(reason: &str) -> Error {
    ComputeProvisioningError::AmiLookupFailed {
        name: AL2023_AMI_NAME.to_string(),
        reason: reason.to_string(),
    }
    .into()
}
