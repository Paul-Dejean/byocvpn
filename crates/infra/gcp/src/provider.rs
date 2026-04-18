use std::str::FromStr;

use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams, SpawnStep,
        TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use log::*;
use serde_json::Value;

use crate::{
    auth::parse_credentials_from_service_account_json, client::GcpClient, instance, network,
};

pub struct GcpProviderConfig {
    pub service_account_json: String,
}

pub struct GcpProvider {
    client: GcpClient,
}

impl GcpProvider {
    pub fn new(config: GcpProviderConfig) -> Result<Self> {
        let (credentials, project_id) =
            parse_credentials_from_service_account_json(&config.service_account_json)?;
        let client = GcpClient::new(credentials, project_id);
        Ok(Self { client })
    }
}

pub enum GcpSpawnStepId {
    SetupApi,
    SetupVpc,
    SetupFirewall,
    RegionSubnet,
    Launch,
    WireguardReady,
}

impl GcpSpawnStepId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SetupApi => "setup_api",
            Self::SetupVpc => "setup_vpc",
            Self::SetupFirewall => "setup_firewall",
            Self::RegionSubnet => "region_subnet",
            Self::Launch => "launch",
            Self::WireguardReady => "wireguard_ready",
        }
    }
}

impl FromStr for GcpSpawnStepId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "setup_api" => Ok(Self::SetupApi),
            "setup_vpc" => Ok(Self::SetupVpc),
            "setup_firewall" => Ok(Self::SetupFirewall),
            "region_subnet" => Ok(Self::RegionSubnet),
            "launch" => Ok(Self::Launch),
            "wireguard_ready" => Ok(Self::WireguardReady),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl CloudProvider for GcpProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Gcp
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: GcpSpawnStepId::SetupApi.as_str().into(),
                label: "Enabling Compute Engine API".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::SetupVpc.as_str().into(),
                label: "Creating VPC network".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::SetupFirewall.as_str().into(),
                label: "Creating firewall rules".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::RegionSubnet.as_str().into(),
                label: "Creating regional subnet".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::Launch.as_str().into(),
                label: "Launching Compute Engine instance".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::WireguardReady.as_str().into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<GcpSpawnStepId>() else {
            return Ok(());
        };
        match step {
            GcpSpawnStepId::SetupApi => {
                network::ensure_compute_api_enabled(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::SetupVpc => {
                network::get_or_create_vpc(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::SetupFirewall => {
                network::get_or_create_firewall(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::RegionSubnet => {
                network::get_or_create_subnet(&self.client, region).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    async fn setup(&self) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::get_or_create_vpc(&self.client).await?;
        network::get_or_create_firewall(&self.client).await?;
        info!("GCP setup complete (VPC + firewall).");
        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::get_or_create_vpc(&self.client).await?;
        network::get_or_create_firewall(&self.client).await?;
        network::get_or_create_subnet(&self.client, region).await?;
        info!("GCP region {} enabled.", region);
        Ok(())
    }

    fn get_provision_account_steps(&self) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: GcpSpawnStepId::SetupApi.as_str().into(),
                label: "Enabling Compute Engine API".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::SetupVpc.as_str().into(),
                label: "Creating VPC network".into(),
            },
            SpawnStep {
                id: GcpSpawnStepId::SetupFirewall.as_str().into(),
                label: "Creating firewall rules".into(),
            },
        ]
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        self.run_spawn_step(step_id, "").await
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![SpawnStep {
            id: GcpSpawnStepId::RegionSubnet.as_str().into(),
            label: "Creating regional subnet".into(),
        }]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let subnet_self_link = network::get_or_create_subnet(&self.client, params.region)
            .await
            .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                reason: error.to_string(),
            })?;
        debug!(
            "Resolved GCP subnet {} for spawn in {}",
            subnet_self_link, params.region
        );

        let image_self_link = network::get_ubuntu_image_self_link(&self.client).await?;
        debug!("Resolved GCP Ubuntu image {}", image_self_link);

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
