use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};

use crate::{commands::setup::Region, error::Result};

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

    fn provision_account_steps(&self) -> Vec<SpawnStep>;
    async fn run_provision_account_step(&self, step_id: &str) -> Result<()>;

    fn enable_region_steps(&self, region: &str) -> Vec<SpawnStep>;
    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum CloudProviderName {
    Aws,
    Azure,
    Gcp,
    #[strum(serialize = "oracle", serialize = "oci")]
    Oracle,
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
    pub storage_gb: f64,
    pub storage_rate_per_gb_month: f64,
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
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionAccountJob {
    pub job_id: String,
    pub steps: Vec<SpawnStep>,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionAccountProgressEvent {
    pub job_id: String,
    pub step_id: String,
    pub status: SpawnStepStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionAccountCompleteEvent {
    pub job_id: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableRegionJob {
    pub job_id: String,
    pub steps: Vec<SpawnStep>,
    pub region: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableRegionProgressEvent {
    pub job_id: String,
    pub step_id: String,
    pub status: SpawnStepStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableRegionCompleteEvent {
    pub job_id: String,
    pub region: String,
    pub provider: CloudProviderName,
}
