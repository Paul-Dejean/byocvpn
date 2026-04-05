use aws_sdk_ssm::error::{ProvideErrorMetadata, SdkError};
use byocvpn_core::error::Error;

pub fn sdk_error_message<E>(error: &SdkError<E>) -> String
where
    E: ProvideErrorMetadata,
{
    match error {
        SdkError::ServiceError(service_error) => {
            let code = service_error.err().code().unwrap_or("UnknownError");
            let message = service_error.err().message().unwrap_or("no message");
            format!("{}: {}", code, message)
        }
        SdkError::TimeoutError(_) => "request timed out".to_string(),
        SdkError::DispatchFailure(error) => format!("dispatch failure: {:?}", error),
        other => format!("{}", other),
    }
}

pub fn map_aws_error<E>(operation_name: &'static str, sdk_error: SdkError<E>) -> Error
where
    E: std::error::Error + Send + Sync + 'static + ProvideErrorMetadata,
{
    match sdk_error {
        SdkError::ServiceError(service_error) => {
            let error = service_error.into_err();
            let code_string = error.code().unwrap_or_default();
            let message_string = error.message().unwrap_or_default().to_string();

            match code_string {
                "UnauthorizedOperation" => Error::Authentication,
                "AccessDeniedException" => Error::Authorization {
                    operation: operation_name.to_string(),
                },
                "ThrottlingException" | "RequestLimitExceeded" => Error::Quota,
                _ => Error::Unknown {
                    operation_name: operation_name.to_string(),
                    detail: message_string,
                },
            }
        }

        SdkError::TimeoutError(_) | SdkError::DispatchFailure(_) => Error::Transient {
            operation_name: operation_name.to_string(),
        },

        other => Error::Unknown {
            operation_name: operation_name.to_string(),
            detail: other.to_string(),
        },
    }
}
