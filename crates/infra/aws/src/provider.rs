use std::{collections::HashMap, str::FromStr};

use async_trait::async_trait;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ec2::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_ssm::Client as SsmClient;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, PermissionStatus, SpawnInstanceParams,
        SpawnStep, TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use log::*;
use serde_json::Value;

use crate::constants::{
    IPV4_ALL_CIDR, SECURITY_GROUP_NAME, SUBNET_CIDR_BLOCK, SUBNET_NAME, VPC_CIDR_BLOCK, VPC_NAME,
};
use crate::{config, instance, network};

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

fn is_dry_run_authorized<T, E>(operation: &str, result: std::result::Result<T, SdkError<E>>) -> bool
where
    E: ProvideErrorMetadata,
{
    match result {
        Ok(_) => {
            info!("permission check {operation}: authorized (request succeeded)");
            true
        }
        Err(SdkError::ServiceError(service_error)) => {
            let error_code = service_error.err().code().unwrap_or("unknown");
            let error_message = service_error.err().message().unwrap_or("");
            let authorized = is_authorized_error_code(service_error.err().code());
            if authorized {
                info!(
                    "permission check {operation}: authorized (code={error_code}, message={error_message})"
                );
            } else {
                warn!(
                    "permission check {operation}: denied (code={error_code}, message={error_message})"
                );
            }
            authorized
        }
        Err(other_error) => {
            warn!(
                "permission check {operation}: inconclusive (code={:?})",
                other_error.code()
            );
            false
        }
    }
}

fn is_authorized_error_code(code: Option<&str>) -> bool {
    code == Some("DryRunOperation")
}

async fn get_default_vpc_id(ec2_client: &Ec2Client) -> Option<String> {
    let response = ec2_client
        .describe_vpcs()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("isDefault")
                .values("true")
                .build(),
        )
        .send()
        .await
        .ok()?;
    response
        .vpcs()
        .first()
        .and_then(|vpc| vpc.vpc_id())
        .map(|vpc_id| vpc_id.to_string())
}

async fn get_default_security_group_id(ec2_client: &Ec2Client) -> Option<String> {
    let response = ec2_client
        .describe_security_groups()
        .filters(
            aws_sdk_ec2::types::Filter::builder()
                .name("group-name")
                .values("default")
                .build(),
        )
        .send()
        .await
        .ok()?;
    response
        .security_groups()
        .first()
        .and_then(|security_group| security_group.group_id())
        .map(|group_id| group_id.to_string())
}

pub enum AwsSpawnStepId {
    SetupVpc,
    SetupIgw,
    RegionSubnets,
    RegionSecurityGroup,
    SetupNetwork,
    LaunchingInstance,
    WireguardReady,
}

impl AwsSpawnStepId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SetupVpc => "setup_vpc",
            Self::SetupIgw => "setup_igw",
            Self::RegionSubnets => "region_subnets",
            Self::RegionSecurityGroup => "region_security_group",
            Self::SetupNetwork => "setup_network",
            Self::LaunchingInstance => "launch",
            Self::WireguardReady => "wireguard_ready",
        }
    }
}

impl FromStr for AwsSpawnStepId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "setup_vpc" => Ok(Self::SetupVpc),
            "setup_igw" => Ok(Self::SetupIgw),
            "region_subnets" => Ok(Self::RegionSubnets),
            "region_security_group" => Ok(Self::RegionSecurityGroup),
            "setup_network" => Ok(Self::SetupNetwork),
            "launch" => Ok(Self::LaunchingInstance),
            "wireguard_ready" => Ok(Self::WireguardReady),
            _ => Err(()),
        }
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

        let ec2_run_instances = match config::get_al2023_ami(&ssm_client).await {
            Ok(image_id) => is_dry_run_authorized(
                "ec2:RunInstances",
                ec2_client
                    .run_instances()
                    .image_id(image_id)
                    .max_count(1)
                    .min_count(1)
                    .dry_run(true)
                    .send()
                    .await,
            ),
            Err(error) => {
                warn!(
                    "permission check ec2:RunInstances: could not resolve AMI ({error}); failing"
                );
                false
            }
        };

        let ec2_terminate_instances = is_dry_run_authorized(
            "ec2:TerminateInstances",
            ec2_client
                .terminate_instances()
                .instance_ids("i-0ba8dd7fe03dfbb57")
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_create_vpc = is_dry_run_authorized(
            "ec2:CreateVpc",
            ec2_client
                .create_vpc()
                .cidr_block(VPC_CIDR_BLOCK)
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_create_subnet = is_dry_run_authorized(
            "ec2:CreateSubnet",
            ec2_client
                .create_subnet()
                .vpc_id("vpc-12345678")
                .cidr_block(SUBNET_CIDR_BLOCK)
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_create_security_group = is_dry_run_authorized(
            "ec2:CreateSecurityGroup",
            ec2_client
                .create_security_group()
                .group_name(SECURITY_GROUP_NAME)
                .description("permission check")
                .vpc_id("vpc-12345678")
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_create_tags = match get_default_vpc_id(&ec2_client).await {
            Some(vpc_id) => is_dry_run_authorized(
                "ec2:CreateTags",
                ec2_client
                    .create_tags()
                    .resources(vpc_id)
                    .tags(
                        aws_sdk_ec2::types::Tag::builder()
                            .key("Name")
                            .value(VPC_NAME)
                            .build(),
                    )
                    .dry_run(true)
                    .send()
                    .await,
            ),
            None => {
                warn!(
                    "permission check ec2:CreateTags: no default VPC found; treating as authorized"
                );
                true
            }
        };

        let security_group_ingress_permission = aws_sdk_ec2::types::IpPermission::builder()
            .ip_protocol("tcp")
            .from_port(22)
            .to_port(22)
            .ip_ranges(
                aws_sdk_ec2::types::IpRange::builder()
                    .cidr_ip(IPV4_ALL_CIDR)
                    .build(),
            )
            .build();

        let default_security_group_id = get_default_security_group_id(&ec2_client).await;

        let ec2_authorize_security_group_ingress = match &default_security_group_id {
            Some(group_id) => is_dry_run_authorized(
                "ec2:AuthorizeSecurityGroupIngress",
                ec2_client
                    .authorize_security_group_ingress()
                    .group_id(group_id)
                    .ip_permissions(security_group_ingress_permission.clone())
                    .dry_run(true)
                    .send()
                    .await,
            ),
            None => {
                warn!(
                    "permission check ec2:AuthorizeSecurityGroupIngress: no default security group found; treating as authorized"
                );
                true
            }
        };

        let ec2_revoke_security_group_ingress = match &default_security_group_id {
            Some(group_id) => is_dry_run_authorized(
                "ec2:RevokeSecurityGroupIngress",
                ec2_client
                    .revoke_security_group_ingress()
                    .group_id(group_id)
                    .ip_permissions(security_group_ingress_permission)
                    .dry_run(true)
                    .send()
                    .await,
            ),
            None => {
                warn!(
                    "permission check ec2:RevokeSecurityGroupIngress: no default security group found; treating as authorized"
                );
                true
            }
        };

        let ec2_describe_instances = is_dry_run_authorized(
            "ec2:DescribeInstances",
            ec2_client.describe_instances().dry_run(true).send().await,
        );

        let ec2_describe_vpcs = is_dry_run_authorized(
            "ec2:DescribeVpcs",
            ec2_client.describe_vpcs().dry_run(true).send().await,
        );

        let ec2_describe_subnets = is_dry_run_authorized(
            "ec2:DescribeSubnets",
            ec2_client.describe_subnets().dry_run(true).send().await,
        );

        let ec2_describe_security_groups = is_dry_run_authorized(
            "ec2:DescribeSecurityGroups",
            ec2_client
                .describe_security_groups()
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_describe_availability_zones = is_dry_run_authorized(
            "ec2:DescribeAvailabilityZones",
            ec2_client
                .describe_availability_zones()
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_describe_regions = is_dry_run_authorized(
            "ec2:DescribeRegions",
            ec2_client.describe_regions().dry_run(true).send().await,
        );

        let ec2_create_internet_gateway = is_dry_run_authorized(
            "ec2:CreateInternetGateway",
            ec2_client
                .create_internet_gateway()
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_describe_internet_gateways = is_dry_run_authorized(
            "ec2:DescribeInternetGateways",
            ec2_client
                .describe_internet_gateways()
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_attach_internet_gateway = is_dry_run_authorized(
            "ec2:AttachInternetGateway",
            ec2_client
                .attach_internet_gateway()
                .internet_gateway_id("igw-12345678")
                .vpc_id("vpc-12345678")
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_describe_route_tables = is_dry_run_authorized(
            "ec2:DescribeRouteTables",
            ec2_client
                .describe_route_tables()
                .dry_run(true)
                .send()
                .await,
        );

        let ec2_create_route = is_dry_run_authorized(
            "ec2:CreateRoute",
            ec2_client
                .create_route()
                .route_table_id("rtb-12345678")
                .destination_cidr_block(IPV4_ALL_CIDR)
                .gateway_id("igw-12345678")
                .dry_run(true)
                .send()
                .await,
        );

        let ssm_get_parameter_result = ssm_client
            .get_parameter()
            .name(config::AL2023_AMI_SSM_PARAMETER)
            .send()
            .await;
        let ssm_get_parameter = match &ssm_get_parameter_result {
            Ok(_) => {
                info!("permission check ssm:GetParameter: authorized (request succeeded)");
                true
            }
            Err(error) => {
                warn!(
                    "permission check ssm:GetParameter: denied (code={:?}, message={:?})",
                    error.code(),
                    error.message()
                );
                false
            }
        };

        let permissions = [
            ("ec2:RunInstances", ec2_run_instances),
            ("ec2:TerminateInstances", ec2_terminate_instances),
            ("ec2:CreateVpc", ec2_create_vpc),
            ("ec2:CreateSubnet", ec2_create_subnet),
            ("ec2:CreateSecurityGroup", ec2_create_security_group),
            ("ec2:CreateTags", ec2_create_tags),
            (
                "ec2:AuthorizeSecurityGroupIngress",
                ec2_authorize_security_group_ingress,
            ),
            (
                "ec2:RevokeSecurityGroupIngress",
                ec2_revoke_security_group_ingress,
            ),
            ("ec2:DescribeInstances", ec2_describe_instances),
            ("ec2:DescribeVpcs", ec2_describe_vpcs),
            ("ec2:DescribeSubnets", ec2_describe_subnets),
            ("ec2:DescribeSecurityGroups", ec2_describe_security_groups),
            (
                "ec2:DescribeAvailabilityZones",
                ec2_describe_availability_zones,
            ),
            (
                "ec2:DescribeInternetGateways",
                ec2_describe_internet_gateways,
            ),
            ("ec2:CreateInternetGateway", ec2_create_internet_gateway),
            ("ec2:AttachInternetGateway", ec2_attach_internet_gateway),
            ("ec2:DescribeRouteTables", ec2_describe_route_tables),
            ("ec2:CreateRoute", ec2_create_route),
            ("ec2:DescribeRegions", ec2_describe_regions),
            ("ssm:GetParameter", ssm_get_parameter),
        ]
        .into_iter()
        .map(|(permission, granted)| PermissionStatus {
            permission: permission.to_string(),
            granted,
        })
        .collect::<Vec<_>>();

        let value = serde_json::to_value(&permissions).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            }
        })?;
        Ok(value)
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
