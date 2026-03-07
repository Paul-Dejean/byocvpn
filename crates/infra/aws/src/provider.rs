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
use serde::Serialize;
use serde_json::Value;

use crate::{config, instance, network};

pub struct AwsProvider {
    config: AwsProviderConfig,
}

// A good name for the struct would be AwsProviderConfig, as it clearly indicates that it holds configuration for creating an AwsProvider.

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

#[derive(Serialize)]
pub struct AwsPermissionsResult {
    pub ec2_run_instances: bool,
    pub ec2_terminate_instances: bool,
    pub ec2_create_vpc: bool,
    pub ec2_create_subnet: bool,
    pub ec2_create_security_group: bool,
    pub ec2_create_tags: bool,
    pub ec2_authorize_security_group_ingress: bool,
    pub ec2_describe_instances: bool,
    pub ec2_describe_vpcs: bool,
    pub ec2_describe_subnets: bool,
    pub ec2_describe_security_groups: bool,
    pub ec2_describe_availability_zones: bool,
    pub ec2_create_internet_gateway: bool,
    pub ssm_get_parameter: bool,
}

#[async_trait]
impl CloudProvider for AwsProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Aws
    }

    fn spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: "setup_vpc".into(),
                label: "Creating VPC".into(),
            },
            SpawnStep {
                id: "setup_igw".into(),
                label: "Creating internet gateway".into(),
            },
            SpawnStep {
                id: "region_subnets".into(),
                label: "Creating subnets".into(),
            },
            SpawnStep {
                id: "region_security_group".into(),
                label: "Configuring security group".into(),
            },
            SpawnStep {
                id: "launch".into(),
                label: "Launching EC2 instance".into(),
            },
            SpawnStep {
                id: "wireguard_ready".into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn verify_permissions(&self) -> Result<Value> {
        let ec2_client = self.create_ec2_client(None).await;
        let ssm_client = self.create_ssm_client(None).await;
        let ec2_run_instances = ec2_client
            .run_instances()
            .max_count(1)
            .min_count(1)
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_terminate_instances = ec2_client
            .terminate_instances()
            .instance_ids("i-1234567890abcdef0")
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_create_vpc = ec2_client
            .create_vpc()
            .cidr_block("10.0.0.0/16")
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_create_subnet = ec2_client
            .create_subnet()
            .vpc_id("vpc-12345678")
            .cidr_block("10.0.1.0/24")
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_create_security_group = ec2_client
            .create_security_group()
            .group_name("byocvpn-security-group")
            .description("Test SG")
            .vpc_id("vpc-12345678")
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_create_tags = ec2_client
            .create_tags()
            .resources("vpc-12345678")
            .tags(
                aws_sdk_ec2::types::Tag::builder()
                    .key("Name")
                    .value("test")
                    .build(),
            )
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_authorize_security_group_ingress = ec2_client
            .authorize_security_group_ingress()
            .group_id("sg-12345678")
            .ip_permissions(
                aws_sdk_ec2::types::IpPermission::builder()
                    .ip_protocol("tcp")
                    .from_port(22)
                    .to_port(22)
                    .ip_ranges(
                        aws_sdk_ec2::types::IpRange::builder()
                            .cidr_ip("0.0.0.0/0")
                            .build(),
                    )
                    .build(),
            )
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_describe_instances = ec2_client
            .describe_instances()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_describe_vpcs = ec2_client
            .describe_vpcs()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_describe_subnets = ec2_client
            .describe_subnets()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_describe_security_groups = ec2_client
            .describe_security_groups()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_describe_availability_zones = ec2_client
            .describe_availability_zones()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ec2_create_internet_gateway = ec2_client
            .create_internet_gateway()
            .dry_run(true)
            .send()
            .await
            .is_ok();

        let ssm_get_parameter = ssm_client
            .get_parameter()
            .name("test-parameter")
            .send()
            .await
            .is_ok();

        let permissions = AwsPermissionsResult {
            ec2_run_instances,
            ec2_terminate_instances,
            ec2_create_vpc,
            ec2_create_subnet,
            ec2_create_security_group,
            ec2_create_tags,
            ec2_authorize_security_group_ingress,
            ec2_describe_instances,
            ec2_describe_vpcs,
            ec2_describe_subnets,
            ec2_describe_security_groups,
            ec2_describe_availability_zones,
            ec2_create_internet_gateway,

            ssm_get_parameter,
        };

        let value = serde_json::to_value(&permissions).map_err(|e| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: e.to_string(),
            }
        })?;
        Ok(value)
    }

    async fn setup(&self) -> Result<()> {
        let ec2_client = self.create_ec2_client(None).await;
        let existing_vpc_id = network::get_vpc_by_name(&ec2_client, "byocvpn-vpc").await?;
        if existing_vpc_id.is_some() {
            println!("Existing VPC found, skipping creation.");
            return Ok(());
        }
        let vpc_id = network::create_vpc(&ec2_client, "10.0.0.0/16", "byocvpn-vpc").await?;
        let igw_id = network::create_and_attach_igw(&ec2_client, &vpc_id).await?;
        let main_route_table_id = network::find_main_route_table(&ec2_client, &vpc_id).await?;
        network::tag_resource_with_name(
            &ec2_client,
            &main_route_table_id,
            "byocvpn-main-route-table",
        )
        .await?;
        network::tag_resource_with_name(&ec2_client, &igw_id, "byocvpn-igw").await?;
        network::add_igw_routes_to_table(&ec2_client, &main_route_table_id, &igw_id).await?;

        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        self.run_spawn_step("region_subnets", region).await?;
        self.run_spawn_step("region_security_group", region).await?;
        Ok(())
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        match step_id {
            "setup_vpc" => {
                let ec2 = self.create_ec2_client(None).await;
                if network::get_vpc_by_name(&ec2, "byocvpn-vpc")
                    .await?
                    .is_none()
                {
                    network::create_vpc(&ec2, "10.0.0.0/16", "byocvpn-vpc").await?;
                }
                Ok(())
            }
            "setup_igw" => {
                let ec2 = self.create_ec2_client(None).await;
                let vpc_id = network::get_vpc_by_name(&ec2, "byocvpn-vpc").await?.ok_or(
                    NetworkProvisioningError::VpcNotFound {
                        vpc_name: "byocvpn-vpc".to_string(),
                    },
                )?;
                // Look for an IGW already attached to our VPC (idempotent).
                let resp = ec2
                    .describe_internet_gateways()
                    .filters(
                        aws_sdk_ec2::types::Filter::builder()
                            .name("attachment.vpc-id")
                            .values(&vpc_id)
                            .build(),
                    )
                    .send()
                    .await
                    .map_err(
                        |e| NetworkProvisioningError::InternetGatewayOperationFailed {
                            reason: e.to_string(),
                        },
                    )?;
                let igw_id = if let Some(igw) = resp.internet_gateways().first() {
                    igw.internet_gateway_id().unwrap_or_default().to_string()
                } else {
                    network::create_and_attach_igw(&ec2, &vpc_id).await?
                };
                let rt_id = network::find_main_route_table(&ec2, &vpc_id).await?;
                network::tag_resource_with_name(&ec2, &rt_id, "byocvpn-main-route-table").await?;
                network::tag_resource_with_name(&ec2, &igw_id, "byocvpn-igw").await?;
                network::add_igw_routes_to_table(&ec2, &rt_id, &igw_id).await?;
                Ok(())
            }
            "region_subnets" => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                let vpc_id = network::get_vpc_by_name(&ec2, "byocvpn-vpc").await?.ok_or(
                    NetworkProvisioningError::VpcNotFound {
                        vpc_name: "byocvpn-vpc".to_string(),
                    },
                )?;
                let vpc_ipv6_cidr = network::get_vpc_ipv6_block(&ec2, &vpc_id).await?;
                let azs = network::list_availability_zones(&ec2).await?;
                let subnets = network::get_subnets_in_vpc(&ec2, &vpc_id).await?;
                for (i, az) in azs.iter().enumerate() {
                    let subnet_name = format!("byocvpn-subnet-{az}");
                    let already_exists = subnets.iter().any(|s| {
                        s.tags()
                            .iter()
                            .any(|t| t.key() == Some("Name") && t.value() == Some(&subnet_name))
                    });
                    if already_exists {
                        continue;
                    }
                    let cidr = format!("10.0.{}.0/24", i);
                    let ipv6_cidr = network::carve_ipv6_subnet(&vpc_ipv6_cidr, i as u8)?;
                    let subnet_id =
                        network::create_subnet(&ec2, &vpc_id, &cidr, &ipv6_cidr, az, &subnet_name)
                            .await?;
                    network::enable_auto_ip_assign(&ec2, &subnet_id).await?;
                }
                Ok(())
            }
            "region_security_group" => {
                let ec2 = self.create_ec2_client(Some(region.to_string())).await;
                let vpc_id = network::get_vpc_by_name(&ec2, "byocvpn-vpc").await?.ok_or(
                    NetworkProvisioningError::VpcNotFound {
                        vpc_name: "byocvpn-vpc".to_string(),
                    },
                )?;
                let existing =
                    network::get_security_group_by_name(&ec2, "byocvpn-security-group").await?;
                if existing.is_none() {
                    network::create_security_group(
                        &ec2,
                        &vpc_id,
                        "byocvpn-security-group",
                        "BYOC VPN server",
                    )
                    .await?;
                }
                Ok(())
            }
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
        let instances = futures::future::join_all(regions.iter().map(|region| async move {
            let ec2_client = self.create_ec2_client(Some(region.name.clone())).await;
            instance::list_instances_in_region(&ec2_client, &region.name).await
        }))
        .await;
        return instances
            .into_iter()
            .collect::<Result<Vec<Vec<InstanceInfo>>>>()
            .map(|lists| lists.into_iter().flatten().collect());
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let ec2_client = self.create_ec2_client(None).await;
        let regions_map = HashMap::from([
            ("us", "United States"),
            ("eu", "Europe"),
            ("ap", "Asia Pacific"),
            ("sa", "South America"),
            ("ca", "Canada"),
            ("me", "Middle East"),
            ("af", "Africa"),
        ]);
        println!("Fetching regions...");

        let regions = ec2_client
            .describe_regions()
            // .all_regions(true)
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

        println!("Fetched regions: {:?}", regions);
        Ok(regions)
    }
}
