use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, JobStep, PermissionStatus, PricingInfo,
        Region, SpawnInstanceParams, TerminateInstanceParams,
    },
    error::Result,
    jobs::build_unknown_step_error,
};

use crate::{
    auth::create_credential,
    client::AzureClient,
    credentials::AzureCredentials,
    instance, network, permissions, pricing, regions,
    steps::{self, AzureStepId, parse_step_id},
};

pub struct AzureProvider {
    client: AzureClient,
}

impl AzureProvider {
    pub fn new(credentials: AzureCredentials) -> Result<Self> {
        let credential = create_credential(
            &credentials.tenant_id,
            &credentials.application_id,
            &credentials.secret_value,
        )?;
        let client = AzureClient::new(credential, credentials.subscription_id)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CloudProvider for AzureProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        Ok(permissions::verify_permissions(&self.client).await?)
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        Ok(regions::list_regions(&self.client).await?)
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => Ok(instance::list_instances(&self.client, region_name).await?),
            None => Ok(instance::list_all_instances(&self.client).await?),
        }
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let region_network = network::ensure_region_network(&self.client, params.region).await?;
        Ok(instance::spawn_instance(&self.client, &region_network, params).await?)
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        Ok(instance::terminate_instance(&self.client, params.instance_id).await?)
    }

    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo> {
        Ok(pricing::fetch_pricing(region, instance_type).await?)
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        steps::build_provision_account_steps()
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        Err(build_unknown_step_error(CloudProviderName::Azure, step_id))
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_enable_region_steps()
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            AzureStepId::RegionResourceGroup => {
                network::ensure_resource_group(&self.client, region).await?;
                Ok(())
            }
            AzureStepId::RegionVnet => {
                network::ensure_virtual_network_resources(&self.client, region).await?;
                Ok(())
            }
            AzureStepId::SetupNetwork | AzureStepId::Launch | AzureStepId::WireguardReady => {
                Err(build_unknown_step_error(CloudProviderName::Azure, step_id))
            }
        }
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_spawn_steps()
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            AzureStepId::SetupNetwork => {
                network::ensure_region_network(&self.client, region).await?;
                Ok(())
            }
            AzureStepId::Launch | AzureStepId::WireguardReady => Ok(()),
            AzureStepId::RegionResourceGroup | AzureStepId::RegionVnet => {
                Err(build_unknown_step_error(CloudProviderName::Azure, step_id))
            }
        }
    }
}
