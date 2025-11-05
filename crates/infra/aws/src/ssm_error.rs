use aws_sdk_ssm::error::{ProvideErrorMetadata, SdkError};
use byocvpn_core::error::Error;

pub fn map_aws_ssm_error<E>(operation_name: &'static str, sdk_error: SdkError<E>) -> Error
where
    E: std::error::Error + Send + Sync + 'static + ProvideErrorMetadata,
{
    match sdk_error {
        SdkError::ServiceError(service_error) => {
            let error = service_error.into_err();
            let code_string = error.code().unwrap_or_default();
            let message_string = error.message().unwrap_or_default().to_string();

            match code_string {
                "AccessDeniedException" | "UnauthorizedOperation" => {
                    Error::Authorization(message_string)
                }
                "ThrottlingException" | "RequestLimitExceeded" => Error::Quota,
                _ => Error::Unknown {
                    operation_name,
                    detail: message_string,
                    source: Some(Box::new(error)),
                },
            }
        }

        SdkError::TimeoutError(_) | SdkError::DispatchFailure(_) => Error::Transient {
            operation_name,
            source: Box::new(sdk_error),
        },

        other => Error::Unknown {
            operation_name,
            detail: other.to_string(),
            source: Some(Box::new(other)),
        },
    }
}
