use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams, SpawnStep,
        TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use serde_json::Value;

use crate::{auth::credentials_from_service_account_json, client::GcpClient, instance, network};

/// Configuration required to create a `GcpProvider`.
pub struct GcpProviderConfig {
    /// Full service-account JSON key file contents.
    /// The project ID is extracted from the JSON automatically.
    pub service_account_json: String,
}

/// Google Cloud Platform implementation of `CloudProvider`.
pub struct GcpProvider {
    client: GcpClient,
}

impl GcpProvider {
    pub fn new(config: GcpProviderConfig) -> Result<Self> {
        let (credentials, project_id) =
            credentials_from_service_account_json(&config.service_account_json)?;
        let client = GcpClient::new(credentials, project_id);
        Ok(Self { client })
    }
}

#[async_trait]
impl CloudProvider for GcpProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Gcp
    }

    fn spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep { id: "setup_api".into(), label: "Enabling Compute Engine API".into() },
            SpawnStep { id: "setup_vpc".into(), label: "Creating VPC network".into() },
            SpawnStep { id: "setup_firewall".into(), label: "Creating firewall rules".into() },
            SpawnStep { id: "region_subnet".into(), label: "Creating regional subnet".into() },
            SpawnStep { id: "launch".into(), label: "Launching Compute Engine instance".into() },
            SpawnStep { id: "wireguard_ready".into(), label: "Waiting for WireGuard to start".into() },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match step_id {
            "setup_api" => {
                network::ensure_compute_api_enabled(&self.client).await?;
                Ok(())
            }
            "setup_vpc" => {
                network::get_or_create_vpc(&self.client).await?;
                Ok(())
            }
            "setup_firewall" => {
                network::get_or_create_firewall(&self.client).await?;
                Ok(())
            }
            "region_subnet" => {
                network::get_or_create_subnet(&self.client, region).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    /// Create the global VPC and firewall rule. Safe to call multiple times (idempotent).
    async fn setup(&self) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::get_or_create_vpc(&self.client).await?;
        network::get_or_create_firewall(&self.client).await?;
        println!("GCP setup complete (VPC + firewall).");
        Ok(())
    }

    /// Ensure the regional subnet exists. Implicitly calls `setup` first.
    async fn enable_region(&self, region: &str) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::get_or_create_vpc(&self.client).await?;
        network::get_or_create_firewall(&self.client).await?;
        network::get_or_create_subnet(&self.client, region).await?;
        println!("GCP region {} enabled.", region);
        Ok(())
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        // Ensure the region infrastructure exists before spawning.
        let subnet_self_link = network::get_or_create_subnet(&self.client, params.region)
            .await
            .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                reason: error.to_string(),
            })?;

        let image_self_link = network::get_ubuntu_image_self_link(&self.client).await?;

        instance::spawn_instance(
            &self.client,
            &subnet_self_link,
            &image_self_link,
            params.region,
            params,
        )
        .await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        instance::terminate_instance(&self.client, params.instance_id).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => instance::list_instances(&self.client, region_name).await,
            None => instance::list_all_instances(&self.client).await,
        }
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        network::ensure_compute_api_enabled(&self.client).await?;
        let region_list = network::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
