use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    commands::setup::Region,
    error::{ConfigurationError, Error, Result},
};

pub struct SpawnInstanceParams<'a> {
    pub region: &'a str,
    pub server_private_key: &'a str,
    pub client_public_key: &'a str,
}

pub struct TerminateInstanceParams<'a> {
    pub region: &'a str,
    pub instance_id: &'a str,
}

#[async_trait]
pub trait CloudProvider: Send + Sync {
    async fn setup(&self) -> Result<()>;
    async fn verify_permissions(&self) -> Result<Value>;
    async fn enable_region(&self, region: &str) -> Result<()>;
    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo>;
    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()>;
    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>>;
    async fn get_regions(&self) -> Result<Vec<Region>>;
    fn get_provider_name(&self) -> CloudProviderName;

    fn spawn_steps(&self, region: &str) -> Vec<SpawnStep>;

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProviderName {
    Aws,
    Azure,
    Gcp,
    Oracle,
}

impl FromStr for CloudProviderName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "aws" => Ok(CloudProviderName::Aws),
            "azure" => Ok(CloudProviderName::Azure),
            "gcp" => Ok(CloudProviderName::Gcp),
            "oracle" | "oci" => Ok(CloudProviderName::Oracle),
            e => Err(ConfigurationError::UnknownProviderName { name: e.to_string() }.into()),
        }
    }
}

impl Display for CloudProviderName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            CloudProviderName::Aws => "AWS",
            CloudProviderName::Gcp => "GCP",
            CloudProviderName::Azure => "AZURE",
            CloudProviderName::Oracle => "ORACLE",
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceState {
    Running,
    Creating,
    Stopping,
    Stopped,
    Deleting,
    Deleted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub region: String,
    pub state: InstanceState,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
    pub provider: CloudProviderName,
    pub instance_type: String,
    pub launched_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingInfo {
    pub hourly_rate: f64,

    pub ip_hourly_rate: f64,

    pub egress_rate_per_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnStep {
    pub id: String,

    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnJob {
    pub job_id: String,
    pub steps: Vec<SpawnStep>,
    pub region: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SpawnStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnProgressEvent {
    pub job_id: String,
    pub step_id: String,
    pub status: SpawnStepStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnCompleteEvent {
    pub job_id: String,
    pub instance: InstanceInfo,
}
