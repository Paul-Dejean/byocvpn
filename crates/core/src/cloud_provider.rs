use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use async_trait::async_trait;
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
}

#[derive(Debug)]
pub enum CloudProviderName {
    Aws,
    Azure,
    Gcp,
}

impl FromStr for CloudProviderName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "aws" => Ok(CloudProviderName::Aws),
            "azure" => Ok(CloudProviderName::Azure),
            "gcp" => Ok(CloudProviderName::Gcp),
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
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub region: String,
    pub state: String,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
}
