use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, JobStep, PermissionStatus, PricingInfo,
        Region, SpawnInstanceParams, TerminateInstanceParams,
    },
    error::Result,
    jobs::build_unknown_step_error,
};
use futures::future::join_all;
use log::{info, warn};

use crate::{
    client::AwsClient,
    credentials::AwsCredentials,
    image, instance, network, permissions, pricing, regions,
    steps::{self, AwsStepId, parse_step_id},
};

pub struct AwsProvider {
    credentials: AwsCredentials,
}

impl AwsProvider {
    pub const fn new(credentials: AwsCredentials) -> Self {
        Self { credentials }
    }

    async fn build_client(&self, region: Option<&str>) -> AwsClient {
        AwsClient::connect(&self.credentials, region).await
    }

    async fn list_instances_in_all_regions(&self) -> Result<Vec<InstanceInfo>> {
        let regions = self.get_regions().await?;
        let outcomes = join_all(regions.iter().map(|region| async move {
            let client = self.build_client(Some(&region.name)).await;
            match instance::list_instances(&client, &region.name).await {
                Ok(instances) => {
                    info!(
                        "[AWS] Region {}: found {} instances",
                        region.name,
                        instances.len()
                    );
                    instances
                }
                Err(error) => {
                    warn!("[AWS] Skipping region {}: {error}", region.name);
                    Vec::new()
                }
            }
        }))
        .await;
        Ok(outcomes.into_iter().flatten().collect())
    }
}

#[async_trait]
impl CloudProvider for AwsProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Aws
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        let client = self.build_client(None).await;
        permissions::verify_permissions(&client).await
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let client = self.build_client(None).await;
        regions::list_regions(&client).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => {
                let client = self.build_client(Some(region_name)).await;
                instance::list_instances(&client, region_name).await
            }
            None => self.list_instances_in_all_regions().await,
        }
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let client = self.build_client(Some(params.region)).await;
        let region_network = network::ensure_region_network(&client).await?;
        let image_id = image::resolve_boot_image(&client).await?;
        instance::spawn_instance(&client, &region_network, &image_id, params).await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        let client = self.build_client(Some(params.region)).await;
        instance::terminate_instance(&client, params.instance_id).await
    }

    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo> {
        let client = self.build_client(None).await;
        Box::pin(pricing::fetch_pricing(&client, region, instance_type)).await
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        steps::build_provision_account_steps()
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        Err(build_unknown_step_error(CloudProviderName::Aws, step_id))
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_enable_region_steps()
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        let client = self.build_client(Some(region)).await;
        match parse_step_id(step_id)? {
            AwsStepId::SetupVpc => {
                network::ensure_vpc(&client).await?;
            }
            AwsStepId::SetupIgw => {
                let vpc_id = network::ensure_vpc(&client).await?;
                network::ensure_internet_gateway(&client, &vpc_id).await?;
            }
            AwsStepId::RegionSubnets => {
                let vpc_id = network::ensure_vpc(&client).await?;
                network::ensure_subnet(&client, &vpc_id).await?;
            }
            AwsStepId::RegionSecurityGroup => {
                let vpc_id = network::ensure_vpc(&client).await?;
                network::ensure_security_group(&client, &vpc_id).await?;
            }
            AwsStepId::SetupNetwork | AwsStepId::Launch | AwsStepId::WireguardReady => {
                return Err(build_unknown_step_error(CloudProviderName::Aws, step_id));
            }
        }
        Ok(())
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_spawn_steps()
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            AwsStepId::SetupNetwork => {
                let client = self.build_client(Some(region)).await;
                network::ensure_region_network(&client).await?;
                Ok(())
            }
            AwsStepId::Launch | AwsStepId::WireguardReady => Ok(()),
            AwsStepId::SetupVpc
            | AwsStepId::SetupIgw
            | AwsStepId::RegionSubnets
            | AwsStepId::RegionSecurityGroup => {
                Err(build_unknown_step_error(CloudProviderName::Aws, step_id))
            }
        }
    }
}
