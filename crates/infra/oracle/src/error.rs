use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};

use byocvpn_core::{
    cloud_provider::CloudProviderName,
    error::{CloudError, Error as CoreError, ErrorKind},
};
use reqwest::{Error as ReqwestError, Method, StatusCode};
use rsa::{pkcs8::Error as Pkcs8Error, signature::Error as SignatureError};
use serde::Deserialize;
use serde_json::{Error as JsonError, from_str};
use thiserror::Error;
use url::ParseError as UrlParseError;

use crate::state::OciLifecycleState;

pub type Result<T> = StdResult<T, OracleError>;

#[derive(Debug, Error)]
pub enum OracleError {
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
        source: ReqwestError,
    },

    #[error("failed to build the HTTP client")]
    HttpClientBuild(#[source] ReqwestError),

    #[error("invalid request URL {url}")]
    InvalidUrl {
        url: String,
        #[source]
        source: UrlParseError,
    },

    #[error("{request} returned a body that is not valid JSON")]
    InvalidJson {
        request: ApiRequest,
        #[source]
        source: JsonError,
    },

    #[error("{request} request body could not be serialized")]
    InvalidRequestBody {
        request: ApiRequest,
        #[source]
        source: JsonError,
    },

    #[error("invalid private key")]
    InvalidPrivateKey(#[from] Pkcs8Error),

    #[error("request signing failed")]
    RequestSigning(#[from] SignatureError),

    #[error("{resource} is missing {field}")]
    MissingField {
        resource: &'static str,
        field: &'static str,
    },

    #[error("no availability domains found in {region}")]
    NoAvailabilityDomains { region: String },

    #[error("instance {instance_ocid} entered terminal state {lifecycle_state}")]
    InstanceTerminated {
        instance_ocid: String,
        lifecycle_state: OciLifecycleState,
    },

    #[error("instance {instance_ocid} did not reach the running state in time")]
    InstanceNotRunning { instance_ocid: String },

    #[error("region {region} not found")]
    RegionNotFound { region: String },

    #[error("subscription to region {region} did not become ready in time")]
    RegionSubscriptionTimedOut { region: String },

    #[error("base image {image} not found")]
    BaseImageNotFound { image: &'static str },

    #[error("shape {shape} has no known part numbers")]
    UnknownShape { shape: String },

    #[error("price list returned HTTP {status} for part {part_number}")]
    PriceListRejected { part_number: String, status: u16 },

    #[error("no pay-as-you-go USD price found for part {part_number}")]
    PriceUnavailable { part_number: String },

    #[error(transparent)]
    Core(#[from] CoreError),
}

#[derive(Debug, Clone)]
pub struct ApiRequest {
    method: Method,
    url: String,
}

#[derive(Deserialize)]
struct OciErrorResponse {
    code: Option<String>,
    message: Option<String>,
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

    pub fn rejected(&self, status: StatusCode, body: &str) -> OracleError {
        let error_body = from_str::<OciErrorResponse>(body).ok();
        OracleError::Rejected {
            request: self.clone(),
            status: status.as_u16(),
            code: error_body
                .as_ref()
                .and_then(|error_body| error_body.code.clone()),
            message: error_body.and_then(|error_body| error_body.message),
            body: body.to_string(),
        }
    }

    pub fn transport(&self, source: ReqwestError) -> OracleError {
        OracleError::Transport {
            request: self.clone(),
            source,
        }
    }

    pub fn invalid_json(&self, source: JsonError) -> OracleError {
        OracleError::InvalidJson {
            request: self.clone(),
            source,
        }
    }

    pub fn invalid_body(&self, source: JsonError) -> OracleError {
        OracleError::InvalidRequestBody {
            request: self.clone(),
            source,
        }
    }
}

impl CloudError for OracleError {
    fn provider(&self) -> CloudProviderName {
        CloudProviderName::Oracle
    }

    fn kind(&self) -> ErrorKind {
        match self {
            Self::Rejected { status, code, .. } => code
                .as_deref()
                .and_then(classify_oci_code)
                .unwrap_or_else(|| ErrorKind::from_http_status(*status)),
            Self::Transport { .. }
            | Self::InstanceNotRunning { .. }
            | Self::RegionSubscriptionTimedOut { .. } => ErrorKind::Transient,
            Self::InvalidPrivateKey(_) | Self::RequestSigning(_) => ErrorKind::Authentication,
            Self::BaseImageNotFound { .. } | Self::RegionNotFound { .. } => ErrorKind::NotFound,
            Self::HttpClientBuild(_)
            | Self::InvalidUrl { .. }
            | Self::InvalidJson { .. }
            | Self::InvalidRequestBody { .. }
            | Self::Core(_) => ErrorKind::Unknown,
            Self::MissingField { .. }
            | Self::NoAvailabilityDomains { .. }
            | Self::InstanceTerminated { .. }
            | Self::UnknownShape { .. }
            | Self::PriceListRejected { .. }
            | Self::PriceUnavailable { .. } => ErrorKind::Cloud,
        }
    }

    fn operation(&self) -> Option<String> {
        match self {
            Self::Rejected { request, .. }
            | Self::Transport { request, .. }
            | Self::InvalidJson { request, .. }
            | Self::InvalidRequestBody { request, .. } => Some(request.to_string()),
            _ => None,
        }
    }

    fn code(&self) -> Option<String> {
        match self {
            Self::Rejected { code, .. } => code.clone(),
            _ => None,
        }
    }
}

impl From<OracleError> for CoreError {
    fn from(error: OracleError) -> Self {
        match error {
            OracleError::Core(core_error) => core_error,
            other => Self::Cloud(Box::new(other)),
        }
    }
}

impl Display for ApiRequest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{} {}", self.method, self.url)
    }
}

fn classify_oci_code(code: &str) -> Option<ErrorKind> {
    match code {
        "NotAuthorizedOrNotFound" | "NotAllowed" => Some(ErrorKind::Authorization),
        "NotAuthenticated" => Some(ErrorKind::Authentication),
        "TooManyRequests" | "LimitExceeded" | "QuotaExceeded" => Some(ErrorKind::Quota),
        "NotFound" => Some(ErrorKind::NotFound),
        _ => None,
    }
}
