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
    /// Returns the ordered list of deployment steps for this provider and
    /// region. Called synchronously before any cloud API requests so the UI
    /// can render the full step list immediately when spawn begins.
    fn spawn_steps(&self, region: &str) -> Vec<SpawnStep>;
    /// Executes a single named sub-step of the deployment pipeline.
    /// All step ids from [`spawn_steps`] that are not reserved
    /// (`launch`, `wireguard_ready`) are dispatched here.
    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()>;
}

#[derive(Debug)]
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
            e => Err(ConfigurationError::InvalidCloudProvider(e.to_string()).into()),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub region: String,
    pub state: String,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
    pub provider: String,
    pub instance_type: String,
    pub launched_at: Option<DateTime<Utc>>,
}

/// Pricing information for a specific instance type on a cloud provider.
///
/// All rates are in USD. Updated: 2026-03-03.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingInfo {
    /// VM compute cost per hour.
    pub hourly_rate: f64,
    /// Public IP address cost per hour (charged separately by all providers since 2024).
    pub ip_hourly_rate: f64,
    /// Outbound data transfer cost per GB.
    pub egress_rate_per_gb: f64,
}

/// A single step in the instance deployment pipeline.
///
/// Providers return these via [`CloudProvider::spawn_steps`] so the UI can
/// render the full list before any cloud API calls begin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnStep {
    /// Machine-readable identifier the orchestrator uses to match events to steps.
    pub id: String,
    /// Human-readable label shown in the deployment progress UI.
    pub label: String,
}

/// Returned immediately by the `spawn_instance` Tauri command so the UI can
/// render the step list while the background task runs the deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnJob {
    pub job_id: String,
    pub steps: Vec<SpawnStep>,
    pub region: String,
    pub provider: String,
}

/// Step execution state, carried in [`SpawnProgressEvent`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SpawnStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Emitted on the `"spawn-progress"` Tauri event after each step transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnProgressEvent {
    pub job_id: String,
    pub step_id: String,
    pub status: SpawnStepStatus,
    pub error: Option<String>,
}

/// Emitted on the `"spawn-complete"` Tauri event once the instance is fully
/// ready and the WireGuard config has been written to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnCompleteEvent {
    pub job_id: String,
    pub instance: InstanceInfo,
}
