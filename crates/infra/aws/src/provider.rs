use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
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

use crate::constants::{
    SECURITY_GROUP_NAME, SUBNET_CIDR_BLOCK, SUBNET_NAME, VPC_CIDR_BLOCK, VPC_NAME,
};
use crate::spawn_step::AwsSpawnStepId;
use crate::{config, instance, network, permissions};

const INTERNET_GATEWAY_NAME: &str = "byocvpn-igw";
const MAIN_ROUTE_TABLE_NAME: &str = "byocvpn-main-route-table";

pub struct AwsProvider {
    config: AwsProviderConfig,
}

pub struct AwsProviderConfig {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}
impl AwsProvider {
    pub async fn new(config: AwsProviderConfig) -> Self {
        Self { config: config }
    }

    pub async fn create_ec2_client(&self, region: Option<String>) -> Ec2Client {
        let sdk_config = config::get_sdk_config(&self.config, region).await;
        return Ec2Client::new(&sdk_config);
    }
    pub async fn create_ssm_client(&self, region: Option<String>) -> SsmClient {
        let sdk_config = config::get_sdk_config(&self.config, region).await;
        return SsmClient::new(&sdk_config);
    }
}

#[async_trait]
impl CloudProvider for AwsProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Aws
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: AwsSpawnStepId::SetupNetwork.as_str().into(),
                label: "Verifying network infrastructure".into(),
            },
            SpawnStep {
                id: AwsSpawnStepId::LaunchingInstance.as_str().into(),
                label: "Launching EC2 instance".into(),
            },
            SpawnStep {
                id: AwsSpawnStepId::WireguardReady.as_str().into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn verify_permissions(&self) -> Result<Value> {
        let ec2_client = self.create_ec2_client(None).await;
        let ssm_client = self.create_ssm_client(None).await;
        permissions::verify_permissions(&ec2_client, &ssm_client).await
    }

    async fn setup(&self) -> Result<()> {
        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        self.run_spawn_step(AwsSpawnStepId::SetupVpc.as_str(), region)
            .await?;
        self.run_spawn_step(AwsSpawnStepId::SetupIgw.as_str(), region)
            .await?;
        self.run_spawn_step(AwsSpawnStepId::RegionSubnets.as_str(), region)
            .await?;
        self.run_spawn_step(AwsSpawnStepId::RegionSecurityGroup.as_str(), region)
            .await?;
        Ok(())
    }

    fn get_provision_account_steps(&self) -> Vec<SpawnStep> {
        vec![]
    }

    async fn run_provision_account_step(&self, _step_id: &str) -> Result<()> {
        Ok(())
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: AwsSpawnStepId::SetupVpc.as_str().into(),
                label: "Creating VPC".into(),
            },
            SpawnStep {
                id: AwsSpawnStepId::SetupIgw.as_str().into(),
                label: "Creating internet gateway".into(),
            },
            SpawnStep {
                id: AwsSpawnStepId::RegionSubnets.as_str().into(),
                label: "Creating subnets".into(),
            },
            SpawnStep {
                id: AwsSpawnStepId::RegionSecurityGroup.as_str().into(),
                label: "Configuring security group".into(),
            },
        ]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<AwsSpawnStepId>() else {
            return Ok(());
        };
        match step {
            AwsSpawnStepId::SetupVpc => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                network::ensure_vpc(&ec2, VPC_CIDR_BLOCK, VPC_NAME).await?;
                Ok(())
            }
            AwsSpawnStepId::SetupIgw => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                let vpc_id = network::ensure_vpc(&ec2, VPC_CIDR_BLOCK, VPC_NAME).await?;
                network::ensure_internet_gateway(
                    &ec2,
                    &vpc_id,
                    INTERNET_GATEWAY_NAME,
                    MAIN_ROUTE_TABLE_NAME,
                )
                .await?;
                Ok(())
            }
            AwsSpawnStepId::RegionSubnets => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                let vpc_id = network::ensure_vpc(&ec2, VPC_CIDR_BLOCK, VPC_NAME).await?;
                network::ensure_subnet(&ec2, &vpc_id, SUBNET_CIDR_BLOCK, SUBNET_NAME).await?;
                Ok(())
            }
            AwsSpawnStepId::RegionSecurityGroup => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                let vpc_id = network::ensure_vpc(&ec2, VPC_CIDR_BLOCK, VPC_NAME).await?;
                network::ensure_security_group(
                    &ec2,
                    &vpc_id,
                    SECURITY_GROUP_NAME,
                    "BYOC VPN server",
                )
                .await?;
                Ok(())
            }
            AwsSpawnStepId::SetupNetwork => self.enable_region(region).await,
            _ => Ok(()),
        }
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let ec2_client = self
            .create_ec2_client(Some(params.region.to_string()))
            .await;
        let ssm_client = self
            .create_ssm_client(Some(params.region.to_string()))
            .await;
        instance::spawn_instance(
            &ec2_client,
            &ssm_client,
            params.region,
            params.spawn_id,
            params.server_private_key,
            params.client_public_key,
        )
        .await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        let ec2_client = self
            .create_ec2_client(Some(params.region.to_string()))
            .await;
        instance::terminate_instance(&ec2_client, params.instance_id).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        if let Some(region_name) = region {
            let ec2_client = self.create_ec2_client(Some(region_name.to_string())).await;
            return instance::list_instances_in_region(&ec2_client, region_name).await;
        }
        let regions = self.get_regions().await?;
        let results = futures::future::join_all(regions.iter().map(|region| async move {
            info!("Listing instances in region {}", region.name);
            let ec2_client = self.create_ec2_client(Some(region.name.clone())).await;
            let result = instance::list_instances_in_region(&ec2_client, &region.name).await;
            match &result {
                Ok(instances) => info!(
                    "Region {}: found {} instances",
                    region.name,
                    instances.len()
                ),
                Err(error) => warn!("Skipping region {}: {}", region.name, error),
            }
            result
        }))
        .await;
        return Ok(results
            .into_iter()
            .filter_map(|result| result.ok())
            .flatten()
            .collect());
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let ec2_client = self.create_ec2_client(None).await;
        let regions_map = HashMap::from([
            ("us", "North America"),
            ("eu", "Europe"),
            ("ap", "Asia Pacific"),
            ("sa", "South America"),
            ("ca", "North America"),
            ("me", "Middle East"),
            ("af", "Africa"),
            ("il", "Middle East"),
            ("mx", "North America"),
        ]);
        info!("Fetching regions...");

        let regions = ec2_client
            .describe_regions()
            .send()
            .await
            .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            })?
            .regions()
            .iter()
            .filter_map(|region| region.region_name())
            .map(|name| {
                let region_prefix = name.split('-').next().unwrap_or("unknown");
                let country = regions_map
                    .get(region_prefix)
                    .unwrap_or(&"Unknown")
                    .to_string();
                Region {
                    name: name.to_string(),
                    country,
                }
            })
            .collect();

        info!("Fetched regions: {:?}", regions);
        Ok(regions)
    }
}
