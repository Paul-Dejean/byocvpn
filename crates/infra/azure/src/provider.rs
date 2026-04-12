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

use crate::{auth::create_credential, client::AzureClient, instance, network};

pub struct AzureProviderConfig {
    pub subscription_id: String,

    pub tenant_id: String,

    pub client_id: String,

    pub client_secret: String,
}

pub struct AzureProvider {
    client: AzureClient,
}

impl AzureProvider {
    pub fn new(config: AzureProviderConfig) -> Result<Self> {
        let credential =
            create_credential(&config.tenant_id, &config.client_id, &config.client_secret)?;
        let client = AzureClient::new(credential, config.subscription_id);
        Ok(Self { client })
    }
}

pub enum AzureSpawnStepId {
    RegionResourceGroup,
    RegionVnet,
    Launch,
    WireguardReady,
}

impl AzureSpawnStepId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegionResourceGroup => "region_resource_group",
            Self::RegionVnet => "region_vnet",
            Self::Launch => "launch",
            Self::WireguardReady => "wireguard_ready",
        }
    }
}

impl FromStr for AzureSpawnStepId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "region_resource_group" => Ok(Self::RegionResourceGroup),
            "region_vnet" => Ok(Self::RegionVnet),
            "launch" => Ok(Self::Launch),
            "wireguard_ready" => Ok(Self::WireguardReady),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl CloudProvider for AzureProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    fn spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: AzureSpawnStepId::RegionResourceGroup.as_str().into(),
                label: "Creating resource group".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::RegionVnet.as_str().into(),
                label: "Creating VNet and subnet".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::Launch.as_str().into(),
                label: "Launching virtual machine".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::WireguardReady.as_str().into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<AzureSpawnStepId>() else {
            return Ok(());
        };
        match step {
            AzureSpawnStepId::RegionResourceGroup => {
                if network::get_resource_group_by_location(&self.client, region)
                    .await?
                    .is_none()
                {
                    network::create_resource_group(&self.client, region).await?;
                }
                Ok(())
            }
            AzureSpawnStepId::RegionVnet => {
                let nsg_id = network::ensure_nsg(&self.client, region).await?;
                if network::get_vnet(&self.client, region).await?.is_none() {
                    network::create_vnet(&self.client, region).await?;
                }
                if network::get_subnet(&self.client, region).await?.is_none() {
                    network::create_subnet(&self.client, region, &nsg_id)
                        .await
                        .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                            reason: error.to_string(),
                        })?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    async fn setup(&self) -> Result<()> {
        info!("[Azure] setup() — no global resources required.");
        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        network::ensure_region_networking(&self.client, region).await?;
        info!("[Azure] Region '{}' enabled.", region);
        Ok(())
    }

    fn provision_account_steps(&self) -> Vec<SpawnStep> {
        vec![]
    }

    async fn run_provision_account_step(&self, _step_id: &str) -> Result<()> {
        Ok(())
    }

    fn enable_region_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: AzureSpawnStepId::RegionResourceGroup.as_str().into(),
                label: "Creating resource group".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::RegionVnet.as_str().into(),
                label: "Creating VNet and subnet".into(),
            },
        ]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        instance::spawn_instance(&self.client, params.region, params).await
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
        let region_list = network::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
