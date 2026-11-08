use crate::{
    client::AwsClient,
    error::{AwsError, Result},
};

const AL2023_AMI_SSM_PARAMETER: &str =
    "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64";
const AL2023_AMI_NAME: &str = "al2023";
const PARAMETER_NOT_FOUND_CODE: &str = "ParameterNotFound";

pub async fn resolve_boot_image(client: &AwsClient) -> Result<String> {
    let response = match client
        .ssm
        .get_parameter()
        .name(AL2023_AMI_SSM_PARAMETER)
        .send()
        .await
    {
        Ok(response) => response,
        Err(sdk_error) => return Err(describe_lookup_failure(AwsError::from(sdk_error))),
    };

    response
        .parameter()
        .and_then(|parameter| parameter.value())
        .map(ToString::to_string)
        .ok_or(AwsError::AmiLookupFailed {
            name: AL2023_AMI_NAME,
            reason: "parameter has no value",
        })
}

fn describe_lookup_failure(error: AwsError) -> AwsError {
    if error.has_code(PARAMETER_NOT_FOUND_CODE) {
        AwsError::AmiLookupFailed {
            name: AL2023_AMI_NAME,
            reason: "parameter not found",
        }
    } else {
        error
    }
}
