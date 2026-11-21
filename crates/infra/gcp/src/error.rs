use std::fmt::{Display, Formatter, Result as FmtResult};

use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{CloudError, Error as CoreError, ErrorKind},
};
use google_cloud_auth::{build_errors::Error as CredentialsBuildError, errors::CredentialsError};
use google_cloud_compute_v1::errors::OperationError;
use google_cloud_gax::{
    client_builder::Error as ClientBuilderError,
    error::{Error as SdkError, rpc::Code},
};
use reqwest::{Error as HttpError, Method, StatusCode};
use serde::Deserialize;
use serde_json::{Error as JsonError, from_str};
use thiserror::Error;

const INTERNAL_SERVER_ERROR_STATUS: u16 = 500;
const GCP_PROVIDER: CloudProviderName = CloudProviderName::Gcp;

pub type Result<T, E = GcpError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum GcpError {
    #[error("{operation} failed")]
    Sdk {
        operation: &'static str,
        #[source]
        source: SdkError,
    },

    #[error("{operation} did not complete")]
    OperationRejected {
        operation: &'static str,
        #[source]
        source: OperationError,
    },

    #[error("{request} rejected with HTTP {status}: {}", .message.as_deref().unwrap_or(.body.as_str()))]
    Rejected {
        request: ApiRequest,
        status: u16,
        code: Option<String>,
        message: Option<String>,
        body: String,
    },

    #[error("{request} could not be completed")]
    Transport {
        request: ApiRequest,
        #[source]
        source: HttpError,
    },

    #[error("{request} returned a body that is not valid JSON")]
    InvalidJson {
        request: ApiRequest,
        #[source]
        source: JsonError,
    },

    #[error("failed to build the {client_name} client")]
    ClientBuild {
        client_name: &'static str,
        #[source]
        source: ClientBuilderError,
    },

    #[error("service usage operation {operation_url} did not complete in time")]
    ServiceUsageOperationTimedOut { operation_url: String },

    #[error("service usage operation rejected: {message}")]
    ServiceUsageOperationRejected { message: String },

    #[error("enabling {api_service} returned no operation name")]
    MissingOperationName { api_service: String },

    #[error("{resource} is missing {field}")]
    MissingField {
        resource: &'static str,
        field: &'static str,
    },

    #[error("no available /20 CIDR block in 10.0.0.0/8")]
    NoSubnetCidrAvailable,

    #[error("base image {image} not found")]
    BaseImageNotFound { image: &'static str },

    #[error("instance id {instance_id} is not in zone/name form")]
    InvalidInstanceId { instance_id: String },

    #[error("no price found for {machine_type} in {region}: {reason}")]
    PriceUnavailable {
        region: String,
        machine_type: String,
        reason: &'static str,
    },

    #[error("service-account JSON is invalid")]
    InvalidServiceAccountJson(#[source] JsonError),

    #[error("service-account JSON is missing project_id")]
    MissingProjectId,

    #[error("service-account credentials are invalid")]
    InvalidCredentials(#[from] CredentialsBuildError),

    #[error("failed to acquire an access token")]
    TokenAcquisition(#[from] CredentialsError),

    #[error("credentials produced no bearer token")]
    MissingBearerToken,

    #[error(transparent)]
    Core(#[from] CoreError),
}

#[derive(Debug, Clone)]
pub struct ApiRequest {
    method: Method,
    url: String,
}

impl GcpError {
    pub fn client_build(client_name: &'static str) -> impl FnOnce(ClientBuilderError) -> Self {
        move |source| Self::ClientBuild {
            client_name,
            source,
        }
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            Self::Sdk { source, .. } => {
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

impl ApiRequest {
    pub fn new(method: Method, url: &str) -> Self {
        Self {
            method,
            url: url.to_string(),
        }
    }

    pub const fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn rejected(&self, status: StatusCode, body: &str) -> GcpError {
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
            request: self.clone(),
            status: status.as_u16(),
            code,
            message,
            body: body.to_string(),
        }
    }

    pub fn transport(&self, source: HttpError) -> GcpError {
        GcpError::Transport {
            request: self.clone(),
            source,
        }
    }

    pub fn invalid_json(&self, source: JsonError) -> GcpError {
        GcpError::InvalidJson {
            request: self.clone(),
            source,
        }
    }
}

pub trait LabelOperation<T> {
    fn label_operation(self, operation: &'static str) -> Result<T>;
}

impl<T> LabelOperation<T> for Result<T, SdkError> {
    fn label_operation(self, operation: &'static str) -> Result<T> {
        self.map_err(|source| GcpError::Sdk { operation, source })
    }
}

impl<T> LabelOperation<T> for Result<T, OperationError> {
    fn label_operation(self, operation: &'static str) -> Result<T> {
        self.map_err(|source| GcpError::OperationRejected { operation, source })
    }
}

impl CloudError for GcpError {
    fn provider(&self) -> CloudProviderName {
        GCP_PROVIDER
    }

    fn kind(&self) -> ErrorKind {
        match self {
            Self::Sdk { source, .. } => classify_sdk_error(source),
            Self::OperationRejected {
                source: OperationError::Generic(details),
                ..
            } => details
                .details
                .as_ref()
                .and_then(|operation_error| operation_error.errors.first())
                .and_then(|detail| detail.code.as_deref())
                .and_then(classify_gcp_code)
                .or_else(|| {
                    details
                        .status_code
                        .and_then(|status_code| u16::try_from(status_code).ok())
                        .map(ErrorKind::from_http_status)
                })
                .unwrap_or(ErrorKind::Cloud),
            Self::Rejected { status, code, .. } => code
                .as_deref()
                .and_then(classify_gcp_code)
                .unwrap_or_else(|| ErrorKind::from_http_status(*status)),
            Self::Transport { .. } | Self::ServiceUsageOperationTimedOut { .. } => {
                ErrorKind::Transient
            }
            Self::InvalidCredentials(_) | Self::TokenAcquisition(_) | Self::MissingBearerToken => {
                ErrorKind::Authentication
            }
            Self::InvalidServiceAccountJson(_)
            | Self::MissingProjectId
            | Self::InvalidInstanceId { .. } => ErrorKind::Configuration,
            Self::BaseImageNotFound { .. } => ErrorKind::NotFound,
            Self::InvalidJson { .. } | Self::ClientBuild { .. } | Self::Core(_) => {
                ErrorKind::Unknown
            }
            Self::OperationRejected { .. }
            | Self::ServiceUsageOperationRejected { .. }
            | Self::MissingOperationName { .. }
            | Self::MissingField { .. }
            | Self::NoSubnetCidrAvailable
            | Self::PriceUnavailable { .. } => ErrorKind::Cloud,
        }
    }

    fn operation(&self) -> Option<String> {
        match self {
            Self::Sdk { operation, .. } | Self::OperationRejected { operation, .. } => {
                Some((*operation).to_string())
            }
            Self::Rejected { request, .. }
            | Self::Transport { request, .. }
            | Self::InvalidJson { request, .. } => Some(request.to_string()),
            _ => None,
        }
    }

    fn code(&self) -> Option<String> {
        match self {
            Self::Sdk { source, .. } => {
                source.status().map(|status| status.code.name().to_string())
            }
            Self::OperationRejected {
                source: OperationError::Generic(details),
                ..
            } => details
                .details
                .as_ref()
                .and_then(|operation_error| operation_error.errors.first())
                .and_then(|detail| detail.code.clone()),
            Self::Rejected { code, .. } => code.clone(),
            _ => None,
        }
    }
}

impl From<GcpError> for CoreError {
    fn from(error: GcpError) -> Self {
        match error {
            GcpError::Core(core_error) => core_error,
            other => Self::Cloud(Box::new(other)),
        }
    }
}

impl Display for ApiRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{} {}", self.method, self.url)
    }
}

fn classify_sdk_error(source: &SdkError) -> ErrorKind {
    if source.is_authentication() {
        return ErrorKind::Authentication;
    }
    let rpc_status = source.status();
    if let Some(kind) = rpc_status.and_then(|status| classify_gcp_code(status.code.name())) {
        return kind;
    }
    source
        .http_status_code()
        .or_else(|| rpc_status.map(|status| map_rpc_code_to_http_status(status.code)))
        .map_or(ErrorKind::Transient, ErrorKind::from_http_status)
}

fn classify_gcp_code(code: &str) -> Option<ErrorKind> {
    match code {
        "PERMISSION_DENIED" | "forbidden" => Some(ErrorKind::Authorization),
        "UNAUTHENTICATED" | "authError" => Some(ErrorKind::Authentication),
        "RESOURCE_EXHAUSTED" | "QUOTA_EXCEEDED" | "rateLimitExceeded" | "quotaExceeded" => {
            Some(ErrorKind::Quota)
        }
        "NOT_FOUND" | "notFound" => Some(ErrorKind::NotFound),
        _ => None,
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
