use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{CredentialsError, Error, NetworkProvisioningError},
};
use google_cloud_compute_v1::errors::OperationError;
use google_cloud_gax::error::{Error as SdkError, rpc::Code};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::from_str;

const INTERNAL_SERVER_ERROR_STATUS: u16 = 500;

pub fn convert_sdk_error(operation: &str, error: SdkError) -> Error {
    if error.is_authentication() {
        return CredentialsError::TokenAcquisitionFailed {
            provider: "GCP".to_string(),
            reason: error.to_string(),
        }
        .into();
    }

    let rpc_status = error.status();
    let code = rpc_status.map(|status| status.code.name().to_string());
    let message = rpc_status
        .map(|status| status.message.clone())
        .filter(|message| !message.is_empty());
    let raw_body = error
        .http_payload()
        .map(|payload| String::from_utf8_lossy(payload).into_owned())
        .unwrap_or_default();

    let http_status = error
        .http_status_code()
        .or_else(|| rpc_status.map(|status| map_rpc_code_to_http_status(status.code)));
    match http_status {
        Some(http_status) => Error::cloud_api_rejected(
            CloudProviderName::Gcp,
            operation,
            http_status,
            code,
            message,
            &raw_body,
        ),
        None => NetworkProvisioningError::NetworkQueryFailed {
            reason: format!("{operation}: {error}"),
        }
        .into(),
    }
}

pub fn convert_operation_error(operation: &str, error: OperationError) -> Error {
    match error {
        OperationError::Generic(details) => {
            let first_detail = details
                .details
                .as_ref()
                .and_then(|operation_error| operation_error.errors.first());
            let code = first_detail.and_then(|detail| detail.code.clone());
            let message = details
                .message
                .clone()
                .or_else(|| first_detail.and_then(|detail| detail.message.clone()));
            let http_status = details
                .status_code
                .and_then(|status_code| u16::try_from(status_code).ok())
                .unwrap_or(INTERNAL_SERVER_ERROR_STATUS);
            Error::cloud_api_rejected(
                CloudProviderName::Gcp,
                operation,
                http_status,
                code,
                message,
                "",
            )
        }
        other => NetworkProvisioningError::CloudOperationFailed {
            reason: format!("{operation}: {other}"),
        }
        .into(),
    }
}

pub fn build_request_error(method: &str, url: &str, status: StatusCode, body: &str) -> Error {
    let error_body = from_str::<GcpErrorResponse>(body)
        .ok()
        .and_then(|response| response.error);
    let code = error_body.as_ref().and_then(|error_body| {
        error_body.status.clone().or_else(|| {
            error_body
                .errors
                .first()
                .and_then(|detail| detail.reason.clone())
        })
    });
    let message = error_body.and_then(|error_body| error_body.message);
    Error::cloud_api_rejected(
        CloudProviderName::Gcp,
        format!("{method} {url}"),
        status.as_u16(),
        code,
        message,
        body,
    )
}

const fn map_rpc_code_to_http_status(code: Code) -> u16 {
    match code {
        Code::Ok => 200,
        Code::InvalidArgument | Code::FailedPrecondition | Code::OutOfRange => 400,
        Code::Unauthenticated => 401,
        Code::PermissionDenied => 403,
        Code::NotFound => 404,
        Code::AlreadyExists | Code::Aborted => 409,
        Code::ResourceExhausted => 429,
        Code::Cancelled => 499,
        Code::Unimplemented => 501,
        Code::Unavailable => 503,
        Code::DeadlineExceeded => 504,
        _ => INTERNAL_SERVER_ERROR_STATUS,
    }
}

#[derive(Deserialize)]
struct GcpErrorResponse {
    error: Option<GcpErrorBody>,
}

#[derive(Deserialize)]
struct GcpErrorBody {
    message: Option<String>,
    status: Option<String>,
    #[serde(default)]
    errors: Vec<GcpErrorDetail>,
}

#[derive(Deserialize)]
struct GcpErrorDetail {
    reason: Option<String>,
}
