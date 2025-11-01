use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use async_trait::async_trait;

use crate::commands::setup::Region;

#[async_trait]
pub trait CloudProvider: Send + Sync {
    async fn setup(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn verify_permissions(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    async fn enable_region(&self, region: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn spawn_instance(
        &self,
        server_private_key: &str,
        client_public_key: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>>;
    async fn terminate_instance(&self, instance_id: &str)
    -> Result<(), Box<dyn std::error::Error>>;
    async fn list_instances(&self) -> Result<Vec<InstanceInfo>, Box<dyn std::error::Error>>;
    fn get_config_file_name(&self, instance_id: &str)
    -> Result<String, Box<dyn std::error::Error>>;

    async fn get_regions(&self) -> Result<Vec<Region>, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum CloudProviderName {
    Aws,
    Azure,
    Gcp,
}

impl FromStr for CloudProviderName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aws" => Ok(CloudProviderName::Aws),
            "azure" => Ok(CloudProviderName::Azure),
            "gcp" => Ok(CloudProviderName::Gcp),
            _ => Err(format!("Unknown cloud provider type: {}", s)),
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

#[derive(Debug)]
pub struct InstanceInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: String,
    pub public_ip_v4: String,
    pub public_ip_v6: String,
}
