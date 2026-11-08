use std::fmt::Display;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::error::Result;

pub struct SpawnInstanceParams<'a> {
    pub region: &'a str,
    pub server_private_key: &'a str,
    pub client_public_key: &'a str,
    pub spawn_id: &'a str,
}

pub struct TerminateInstanceParams<'a> {
    pub region: &'a str,
    pub instance_id: &'a str,
}

#[async_trait]
pub trait CloudProvider: Send + Sync {
    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>>;
    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo>;
    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()>;
    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>>;
    async fn get_regions(&self) -> Result<Vec<Region>>;
    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo>;
    fn get_provider_name(&self) -> CloudProviderName;

    fn get_spawn_steps(&self, region: &str) -> Vec<JobStep>;
    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()>;

    fn get_provision_account_steps(&self) -> Vec<JobStep>;
    async fn run_provision_account_step(&self, step_id: &str) -> Result<()>;

    fn get_enable_region_steps(&self, region: &str) -> Vec<JobStep>;
    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PermissionStatus {
    pub permission: String,
    pub granted: bool,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display, EnumString,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum CloudProviderName {
    Aws,
    Azure,
    Gcp,
    #[strum(serialize = "oracle", serialize = "oci")]
    Oracle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum InstanceState {
    Spawning,
    Installing,
    Error,
    Running,
    Stopping,
    Stopped,
    Unknown,
}

impl InstanceState {
    pub const fn is_listed(&self) -> bool {
        matches!(
            self,
            Self::Running | Self::Stopping | Self::Stopped | Self::Error
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub region: String,
    pub state: InstanceState,
    pub error_reason: Option<String>,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
    pub provider: CloudProviderName,
    pub instance_type: String,
    pub launched_at: Option<DateTime<Utc>>,
    pub spawn_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Region {
    pub name: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PricingInfo {
    pub hourly_rate: f64,
    pub ip_hourly_rate: f64,
    pub egress_rate_per_gb: f64,
    pub storage_gb: f64,
    pub storage_rate_per_gb_month: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct JobStep {
    pub id: String,
    pub label: String,
}

impl JobStep {
    pub fn new(id: impl Display, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SpawnJob {
    pub job_id: String,
    pub steps: Vec<JobStep>,
    pub region: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum JobStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
