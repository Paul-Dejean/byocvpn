use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{Error as CoreError, ErrorKind},
};
use google_cloud_auth::{build_errors::Error as CredentialsBuildError, errors::CredentialsError};
use google_cloud_compute_v1::errors::OperationError;
use google_cloud_gax::{
    client_builder::Error as ClientBuilderError,
    error::{Error as SdkError, rpc::Code},
};
use reqwest::Error as HttpError;
use serde::Deserialize;
use serde_json::{Error as JsonError, from_str};
use thiserror::Error;

const INTERNAL_SERVER_ERROR_STATUS: u16 = 500;

pub type Result<T, E = GcpError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum GcpError {
    #[error("{operation} failed: {source}")]
    Compute {
        operation: &'static str,
        #[source]
        source: SdkError,
    },

    #[error("{operation} did not complete: {source}")]
    OperationFailed {
        operation: &'static str,
        #[source]
        source: OperationError,
    },

    #[error("{method} {url} rejected with HTTP {status}: {}", .message.as_deref().unwrap_or("no message"))]
    Rejected {
        method: &'static str,
        url: String,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        body: String,
    },

    #[error("request to Google Cloud failed: {0}")]
    Transport(#[from] HttpError),

    #[error("invalid JSON in a Google Cloud response: {0}")]
    Json(#[from] JsonError),

    #[error("failed to build the {client_name} client: {source}")]
    ClientBuild {
        client_name: &'static str,
        #[source]
        source: ClientBuilderError,
    },

    #[error("service usage operation {operation_url} did not complete in time")]
    ServiceUsageOperationTimedOut { operation_url: String },

    #[error("service usage operation failed: {message}")]
    ServiceUsageOperationFailed { message: String },

    #[error("enabling {api_service} returned no operation name")]
    MissingOperationName { api_service: String },

    #[error("{resource} is missing {field}")]
    MissingResourceField {
        resource: &'static str,
        field: &'static str,
    },

    #[error("no available /20 CIDR block in 10.0.0.0/8")]
    NoSubnetCidrAvailable,

    #[error("base image {image} not found")]
    BaseImageNotFound { image: &'static str },

    #[error("instance id {instance_id} is not in zone/name form")]
    InvalidInstanceId { instance_id: String },

    #[error("pricing unavailable for {machine_type} in {region}: {reason}")]
    PricingUnavailable {
        region: String,
        machine_type: String,
        reason: &'static str,
    },

    #[error("service-account JSON is invalid: {0}")]
    InvalidServiceAccountJson(#[source] JsonError),

    #[error("service-account JSON is missing project_id")]
    MissingProjectId,

    #[error("service-account credentials are invalid: {0}")]
    InvalidCredentials(#[from] CredentialsBuildError),

    #[error("failed to acquire an access token: {0}")]
    TokenAcquisition(#[from] CredentialsError),

    #[error("credentials produced no bearer token")]
    MissingBearerToken,

    #[error(transparent)]
    Core(#[from] CoreError),
}

impl GcpError {
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::Compute { source, .. } => {
                source.http_status_code() == Some(404)
                    || source
                        .status()
                        .is_some_and(|status| status.code == Code::NotFound)
            }
            Self::Rejected { status, .. } => *status == 404,
            _ => false,
        }
    }
}

pub trait LabelOperation<T> {
    fn label_operation(self, operation: &'static str) -> Result<T>;
}

impl<T> LabelOperation<T> for Result<T, SdkError> {
    fn label_operation(self, operation: &'static str) -> Result<T> {
        self.map_err(|source| GcpError::Compute { operation, source })
    }
}

impl<T> LabelOperation<T> for Result<T, OperationError> {
    fn label_operation(self, operation: &'static str) -> Result<T> {
        self.map_err(|source| GcpError::OperationFailed { operation, source })
    }
}

impl From<GcpError> for CoreError {
    fn from(error: GcpError) -> Self {
        let kind = match error {
            GcpError::Compute { operation, source } => {
                return convert_compute_error(operation, source);
            }
            GcpError::OperationFailed {
                operation,
                source: OperationError::Generic(details),
            } => {
                let first_detail = details
                    .details
                    .as_ref()
                    .and_then(|operation_error| operation_error.errors.first());
                let code = first_detail.and_then(|detail| detail.code.clone());
                let message = details
                    .message
                    .clone()
                    .or_else(|| first_detail.and_then(|detail| detail.message.clone()));
                let status = details
                    .status_code
                    .and_then(|status_code| u16::try_from(status_code).ok())
                    .unwrap_or(INTERNAL_SERVER_ERROR_STATUS);
                return Self::cloud_api_rejected(
                    CloudProviderName::Gcp,
                    operation,
                    status,
                    code,
                    message,
                    "",
                );
            }
            GcpError::Rejected {
                method,
                url,
                status,
                code,
                message,
                body,
            } => {
                return Self::cloud_api_rejected(
                    CloudProviderName::Gcp,
                    format!("{method} {url}"),
                    status,
                    code,
                    message,
                    &body,
                );
            }
            GcpError::Core(core_error) => return core_error,
            GcpError::Transport(_) | GcpError::ServiceUsageOperationTimedOut { .. } => {
                ErrorKind::Transient
            }
            GcpError::InvalidCredentials(_)
            | GcpError::TokenAcquisition(_)
            | GcpError::MissingBearerToken => ErrorKind::Authentication,
            GcpError::InvalidServiceAccountJson(_)
            | GcpError::MissingProjectId
            | GcpError::InvalidInstanceId { .. } => ErrorKind::Configuration,
            GcpError::BaseImageNotFound { .. } => ErrorKind::NotFound,
            GcpError::Json(_) | GcpError::ClientBuild { .. } => ErrorKind::Unknown,
            GcpError::OperationFailed { .. }
            | GcpError::ServiceUsageOperationFailed { .. }
            | GcpError::MissingOperationName { .. }
            | GcpError::MissingResourceField { .. }
            | GcpError::NoSubnetCidrAvailable
            | GcpError::PricingUnavailable { .. } => ErrorKind::Cloud,
        };
        Self::Provider {
            provider: CloudProviderName::Gcp,
            kind,
            source: Box::new(error),
        }
    }
}

pub fn build_rejected_error(method: &'static str, url: &str, status: u16, body: &str) -> GcpError {
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
    GcpError::Rejected {
        method,
        url: url.to_string(),
        status,
        code,
        message,
        body: body.to_string(),
    }
}

fn convert_compute_error(operation: &'static str, source: SdkError) -> CoreError {
    if source.is_authentication() {
        return CoreError::Provider {
            provider: CloudProviderName::Gcp,
            kind: ErrorKind::Authentication,
            source: Box::new(GcpError::Compute { operation, source }),
        };
    }

    let rpc_status = source.status();
    let code = rpc_status.map(|status| status.code.name().to_string());
    let message = rpc_status
        .map(|status| status.message.clone())
        .filter(|message| !message.is_empty());
    let body = source
        .http_payload()
        .map(|payload| String::from_utf8_lossy(payload).into_owned())
        .unwrap_or_default();
    let http_status = source
        .http_status_code()
        .or_else(|| rpc_status.map(|status| map_rpc_code_to_http_status(status.code)));

    match http_status {
        Some(status) => CoreError::cloud_api_rejected(
            CloudProviderName::Gcp,
            operation,
            status,
            code,
            message,
            &body,
        ),
        None => CoreError::Provider {
            provider: CloudProviderName::Gcp,
            kind: ErrorKind::Transient,
            source: Box::new(GcpError::Compute { operation, source }),
        },
    }
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
