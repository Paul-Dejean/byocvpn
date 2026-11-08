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
    account,
    auth::parse_credentials_from_service_account_json,
    client::GcpClient,
    credentials::GcpCredentials,
    image, instance, network, permissions, pricing, regions,
    steps::{self, GcpStepId, parse_step_id},
};

pub struct GcpProvider {
    client: GcpClient,
}

impl GcpProvider {
    pub async fn connect(credentials: GcpCredentials) -> Result<Self> {
        let (service_account_credentials, project_id) =
            parse_credentials_from_service_account_json(&credentials.service_account_json)?;
        let client = GcpClient::connect(service_account_credentials, project_id).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CloudProvider for GcpProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Gcp
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        permissions::verify_permissions(&self.client).await
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        account::ensure_compute_api_enabled(&self.client).await?;
        regions::list_regions(&self.client).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => instance::list_instances(&self.client, region_name).await,
            None => instance::list_all_instances(&self.client).await,
        }
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let region_network = network::ensure_region_network(&self.client, params.region).await?;
        let image_self_link = image::resolve_boot_image(&self.client).await?;
        instance::spawn_instance(&self.client, &region_network, &image_self_link, params).await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        instance::terminate_instance(&self.client, params.instance_id).await
    }

    async fn fetch_pricing(&self, region: &str, instance_type: &str) -> Result<PricingInfo> {
        pricing::fetch_pricing(&self.client, region, instance_type).await
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        steps::build_provision_account_steps()
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            GcpStepId::SetupApi => {
                account::ensure_compute_api_enabled(&self.client).await?;
                account::ensure_cloud_billing_api_enabled(&self.client).await
            }
            GcpStepId::SetupVpc => {
                network::ensure_vpc(&self.client).await?;
                Ok(())
            }
            GcpStepId::SetupFirewall => network::ensure_firewall_rules(&self.client).await,
            GcpStepId::RegionSubnet
            | GcpStepId::SetupNetwork
            | GcpStepId::Launch
            | GcpStepId::WireguardReady => {
                Err(build_unknown_step_error(CloudProviderName::Gcp, step_id))
            }
        }
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_enable_region_steps()
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            GcpStepId::RegionSubnet => {
                network::ensure_subnet(&self.client, region).await?;
                Ok(())
            }
            GcpStepId::SetupApi
            | GcpStepId::SetupVpc
            | GcpStepId::SetupFirewall
            | GcpStepId::SetupNetwork
            | GcpStepId::Launch
            | GcpStepId::WireguardReady => {
                Err(build_unknown_step_error(CloudProviderName::Gcp, step_id))
            }
        }
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        steps::build_spawn_steps()
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match parse_step_id(step_id)? {
            GcpStepId::SetupNetwork => {
                account::ensure_account_network(&self.client).await?;
                network::ensure_region_network(&self.client, region).await?;
                Ok(())
            }
            GcpStepId::Launch | GcpStepId::WireguardReady => Ok(()),
            GcpStepId::SetupApi
            | GcpStepId::SetupVpc
            | GcpStepId::SetupFirewall
            | GcpStepId::RegionSubnet => {
                Err(build_unknown_step_error(CloudProviderName::Gcp, step_id))
            }
        }
    }
}
