use async_trait::async_trait;
use byocvpn_core::jobs::build_unknown_step_error;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, JobStep, PermissionStatus, PricingInfo,
        SpawnInstanceParams, TerminateInstanceParams,
    },
    commands::setup::Region,
    error::Result,
};
use log::info;
use strum::{Display, EnumString};

use crate::{
    auth::create_credential, client::AzureClient, instance, network_security_group::ensure_nsg,
    permissions, pricing, provider_registration::ensure_providers_registered, regions,
    resource_group::ensure_resource_group, subnet::ensure_subnet, virtual_network::ensure_vnet,
};

pub struct AzureProviderConfig {
    pub subscription_id: String,
    pub tenant_id: String,
    pub application_id: String,
    pub secret_value: String,
}

pub struct AzureProvider {
    client: AzureClient,
}

impl AzureProvider {
    pub fn new(config: AzureProviderConfig) -> Result<Self> {
        let credential = create_credential(
            &config.tenant_id,
            &config.application_id,
            &config.secret_value,
        )?;
        let client = AzureClient::new(credential, config.subscription_id)?;
        Ok(Self { client })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AzureSpawnStepId {
    RegionResourceGroup,
    RegionVnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

#[async_trait]
impl CloudProvider for AzureProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![
            JobStep {
                id: AzureSpawnStepId::SetupNetwork.to_string(),
                label: "Verifying network infrastructure".into(),
            },
            JobStep {
                id: AzureSpawnStepId::Launch.to_string(),
                label: "Launching virtual machine".into(),
            },
            JobStep {
                id: AzureSpawnStepId::WireguardReady.to_string(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let step = step_id
            .parse::<AzureSpawnStepId>()
            .map_err(|_| build_unknown_step_error(CloudProviderName::Azure, step_id))?;
        match step {
            AzureSpawnStepId::RegionResourceGroup => {
                ensure_resource_group(&self.client, region).await
            }
            AzureSpawnStepId::RegionVnet => self.ensure_region_network_resources(region).await,
            AzureSpawnStepId::SetupNetwork => {
                ensure_providers_registered(&self.client).await?;
                ensure_resource_group(&self.client, region).await?;
                self.ensure_region_network_resources(region).await?;
                info!("[Azure] Region '{region}' enabled.");
                Ok(())
            }
            AzureSpawnStepId::Launch | AzureSpawnStepId::WireguardReady => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        permissions::verify_permissions(&self.client).await
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        vec![]
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        Err(build_unknown_step_error(CloudProviderName::Azure, step_id))
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![
            JobStep {
                id: AzureSpawnStepId::RegionResourceGroup.to_string(),
                label: "Creating resource group".into(),
            },
            JobStep {
                id: AzureSpawnStepId::RegionVnet.to_string(),
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

    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo> {
        pricing::fetch_pricing(region, instance_type).await
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let region_list = regions::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}

impl AzureProvider {
    async fn ensure_region_network_resources(&self, region: &str) -> Result<()> {
        let nsg_id = ensure_nsg(&self.client, region).await?;
        ensure_vnet(&self.client, region).await?;
        ensure_subnet(&self.client, region, &nsg_id).await?;
        Ok(())
    }
}
