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
use log::warn;

use crate::{
    account,
    client::OciClient,
    credentials::OracleCredentials,
    image, instance, network, permissions, pricing, regions,
    steps::{self, OracleStepId, parse_step_id},
};

pub struct OracleProvider {
    credentials: OracleCredentials,
}

impl OracleProvider {
    pub const fn new(credentials: OracleCredentials) -> Self {
        Self { credentials }
    }

    fn build_client(&self, region: Option<&str>) -> OciClient {
        let region = region.unwrap_or(&self.credentials.region).to_string();
        OciClient::new(self.credentials.clone(), region)
    }

    async fn list_instances_in_subscribed_regions(&self) -> Result<Vec<InstanceInfo>> {
        let home_client = self.build_client(None);
        let regions = regions::list_subscribed_regions(&home_client).await?;
        let outcomes = join_all(regions.iter().map(|region| async move {
            let client = self.build_client(Some(&region.name));
            match instance::list_instances(&client, &region.name).await {
                Ok(instances) => instances,
                Err(error) => {
                    warn!("[OCI] Skipping region {}: {error}", region.name);
                    Vec::new()
                }
            }
        }))
        .await;
        Ok(outcomes.into_iter().flatten().collect())
    }
}

#[async_trait]
impl CloudProvider for OracleProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Oracle
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        Ok(permissions::verify_permissions())
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let client = self.build_client(None);
        Ok(regions::list_regions(&client).await?)
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => {
                let client = self.build_client(Some(region_name));
                Ok(instance::list_instances(&client, region_name).await?)
            }
            None => self.list_instances_in_subscribed_regions().await,
        }
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let client = self.build_client(Some(params.region));
        let region_network = network::ensure_region_network(&client).await?;
        let image_ocid = image::resolve_boot_image(&client).await?;
        Ok(instance::spawn_instance(&client, &region_network, &image_ocid, params).await?)
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        let client = self.build_client(Some(params.region));
        Ok(instance::terminate_instance(&client, params.instance_id).await?)
    }

    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo> {
        Ok(pricing::fetch_pricing(region, instance_type).await?)
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        steps::build_provision_account_steps()
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        let home_client = self.build_client(None);
        match parse_step_id(step_id)? {
            OracleStepId::SetupVcn => {
                network::ensure_vcn(&home_client).await?;
                Ok(())
            }
            OracleStepId::SetupIgw => {
                let vcn = network::ensure_vcn(&home_client).await?;
                network::ensure_internet_gateway(&home_client, &vcn).await?;
                Ok(())
            }
            OracleStepId::RegionSubscribe
            | OracleStepId::RegionVcn
            | OracleStepId::RegionIgw
            | OracleStepId::RegionSecurityList
            | OracleStepId::RegionSubnet
            | OracleStepId::SetupNetwork
            | OracleStepId::Launch
            | OracleStepId::WireguardReady => {
                Err(build_unknown_step_error(CloudProviderName::Oracle, step_id))
            }
        }
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_enable_region_steps()
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        let client = self.build_client(Some(region));
        match parse_step_id(step_id)? {
            OracleStepId::RegionSubscribe => {
                let home_client = self.build_client(None);
                Ok(account::ensure_region_subscribed(&home_client, region).await?)
            }
            OracleStepId::RegionVcn => {
                network::ensure_vcn(&client).await?;
                Ok(())
            }
            OracleStepId::RegionIgw => {
                let vcn = network::ensure_vcn(&client).await?;
                network::ensure_internet_gateway(&client, &vcn).await?;
                Ok(())
            }
            OracleStepId::RegionSecurityList => {
                let vcn = network::ensure_vcn(&client).await?;
                network::ensure_security_list(&client, &vcn).await?;
                Ok(())
            }
            OracleStepId::RegionSubnet => {
                let vcn = network::ensure_vcn(&client).await?;
                let security_list_id = network::ensure_security_list(&client, &vcn).await?;
                network::ensure_subnet(&client, &vcn, &security_list_id).await?;
                Ok(())
            }
            OracleStepId::SetupVcn
            | OracleStepId::SetupIgw
            | OracleStepId::SetupNetwork
            | OracleStepId::Launch
            | OracleStepId::WireguardReady => {
                Err(build_unknown_step_error(CloudProviderName::Oracle, step_id))
            }
        }
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_spawn_steps()
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            OracleStepId::SetupNetwork => {
                let home_client = self.build_client(None);
                account::ensure_account_network(&home_client).await?;
                account::ensure_region_subscribed(&home_client, region).await?;
                let client = self.build_client(Some(region));
                network::ensure_region_network(&client).await?;
                Ok(())
            }
            OracleStepId::Launch | OracleStepId::WireguardReady => Ok(()),
            OracleStepId::SetupVcn
            | OracleStepId::SetupIgw
            | OracleStepId::RegionSubscribe
            | OracleStepId::RegionVcn
            | OracleStepId::RegionIgw
            | OracleStepId::RegionSecurityList
            | OracleStepId::RegionSubnet => {
                Err(build_unknown_step_error(CloudProviderName::Oracle, step_id))
            }
        }
    }
}
