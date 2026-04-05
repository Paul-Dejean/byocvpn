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
use serde_json::Value;

use crate::{auth::OciCredentials, client::OciClient, instance, network};
use log::*;

pub struct OracleProviderConfig {
    pub tenancy_ocid: String,

    pub user_ocid: String,

    pub fingerprint: String,

    pub private_key_pem: String,

    pub region: String,
}

pub struct OracleProvider {
    config: OracleProviderConfig,
}

impl OracleProvider {
    pub fn new(config: OracleProviderConfig) -> Self {
        Self { config }
    }

    fn make_client(&self, region: Option<&str>) -> OciClient {
        let credentials = OciCredentials {
            tenancy_ocid: self.config.tenancy_ocid.clone(),
            user_ocid: self.config.user_ocid.clone(),
            fingerprint: self.config.fingerprint.clone(),
            private_key_pem: self.config.private_key_pem.clone(),
            region: region.unwrap_or(&self.config.region).to_string(),
        };
        OciClient::new(credentials)
    }

    fn get_compartment_ocid(&self) -> &str {
        &self.config.tenancy_ocid
    }
}

pub enum OracleSpawnStepId {
    SetupVcn,
    SetupIgw,
    RegionSubscribe,
    RegionVcn,
    RegionIgw,
    RegionSecurityList,
    RegionSubnet,
    Launch,
    WireguardReady,
}

impl OracleSpawnStepId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SetupVcn => "setup_vcn",
            Self::SetupIgw => "setup_igw",
            Self::RegionSubscribe => "region_subscribe",
            Self::RegionVcn => "region_vcn",
            Self::RegionIgw => "region_igw",
            Self::RegionSecurityList => "region_security_list",
            Self::RegionSubnet => "region_subnet",
            Self::Launch => "launch",
            Self::WireguardReady => "wireguard_ready",
        }
    }
}

impl FromStr for OracleSpawnStepId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "setup_vcn" => Ok(Self::SetupVcn),
            "setup_igw" => Ok(Self::SetupIgw),
            "region_subscribe" => Ok(Self::RegionSubscribe),
            "region_vcn" => Ok(Self::RegionVcn),
            "region_igw" => Ok(Self::RegionIgw),
            "region_security_list" => Ok(Self::RegionSecurityList),
            "region_subnet" => Ok(Self::RegionSubnet),
            "launch" => Ok(Self::Launch),
            "wireguard_ready" => Ok(Self::WireguardReady),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl CloudProvider for OracleProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Oracle
    }

    fn spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: OracleSpawnStepId::SetupVcn.as_str().into(),
                label: "Creating home VCN".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::SetupIgw.as_str().into(),
                label: "Creating internet gateway".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionSubscribe.as_str().into(),
                label: "Subscribing to region \u{2014} this may take several minutes".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionVcn.as_str().into(),
                label: "Creating regional VCN".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionIgw.as_str().into(),
                label: "Creating regional internet gateway".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionSecurityList.as_str().into(),
                label: "Creating security list".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionSubnet.as_str().into(),
                label: "Creating subnet".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::Launch.as_str().into(),
                label: "Launching Ampere A1 instance".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::WireguardReady.as_str().into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<OracleSpawnStepId>() else {
            return Ok(());
        };
        match step {
            OracleSpawnStepId::SetupVcn => {
                let client = self.make_client(None);
                let compartment = self.get_compartment_ocid();
                if network::get_vcn_by_name(&client, compartment)
                    .await?
                    .is_none()
                {
                    network::create_vcn(&client, compartment).await?;
                }
                Ok(())
            }
            OracleSpawnStepId::SetupIgw => {
                let client = self.make_client(None);
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let igw_id =
                    network::get_or_create_internet_gateway(&client, compartment, &vcn_id).await?;
                network::add_default_route_to_table(
                    &client,
                    &route_table_id,
                    &igw_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            OracleSpawnStepId::RegionSubscribe => {
                let home_client = self.make_client(None);
                network::ensure_region_subscribed(&home_client, self.get_compartment_ocid(), region)
                    .await
            }
            OracleSpawnStepId::RegionVcn => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                if network::get_vcn_by_name(&client, compartment)
                    .await?
                    .is_none()
                {
                    network::create_vcn(&client, compartment).await?;
                }
                Ok(())
            }
            OracleSpawnStepId::RegionIgw => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let igw_id =
                    network::get_or_create_internet_gateway(&client, compartment, &vcn_id).await?;
                network::add_default_route_to_table(
                    &client,
                    &route_table_id,
                    &igw_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            OracleSpawnStepId::RegionSecurityList => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                let (vcn_id, _, ipv6_prefix) = network::get_vcn_by_name(&client, compartment)
                    .await?
                    .ok_or(NetworkProvisioningError::VpcNotFound {
                        vpc_name: "byocvpn-vcn".to_string(),
                    })?;
                network::get_or_create_security_list(&client, compartment, &vcn_id, &ipv6_prefix)
                    .await?;
                Ok(())
            }
            OracleSpawnStepId::RegionSubnet => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::get_vcn_by_name(&client, compartment)
                        .await?
                        .ok_or(NetworkProvisioningError::VpcNotFound {
                            vpc_name: "byocvpn-vcn".to_string(),
                        })?;
                let security_list_id = network::get_or_create_security_list(
                    &client,
                    compartment,
                    &vcn_id,
                    &ipv6_prefix,
                )
                .await?;
                let subnet_id =
                    match network::get_subnet_by_name(&client, compartment, &vcn_id).await? {
                        Some((existing_id, _)) => existing_id,
                        None => {
                            network::create_subnet(
                                &client,
                                compartment,
                                &vcn_id,
                                &security_list_id,
                                &route_table_id,
                                &ipv6_prefix,
                            )
                            .await?
                        }
                    };
                network::ensure_subnet_security_list(&client, &subnet_id, &security_list_id)
                    .await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    async fn setup(&self) -> Result<()> {
        self.run_spawn_step(OracleSpawnStepId::SetupVcn.as_str(), "")
            .await?;
        self.run_spawn_step(OracleSpawnStepId::SetupIgw.as_str(), "")
            .await?;
        Ok(())
    }

    fn provision_account_steps(&self) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: OracleSpawnStepId::SetupVcn.as_str().into(),
                label: "Creating home VCN".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::SetupIgw.as_str().into(),
                label: "Creating internet gateway".into(),
            },
        ]
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        self.run_spawn_step(step_id, "").await
    }

    fn enable_region_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: OracleSpawnStepId::RegionSubscribe.as_str().into(),
                label: "Subscribing to region \u{2014} this may take several minutes".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionVcn.as_str().into(),
                label: "Creating regional VCN".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionIgw.as_str().into(),
                label: "Creating regional internet gateway".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionSecurityList.as_str().into(),
                label: "Creating security list".into(),
            },
            SpawnStep {
                id: OracleSpawnStepId::RegionSubnet.as_str().into(),
                label: "Creating subnet".into(),
            },
        ]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        let compartment_ocid = self.get_compartment_ocid();

        let home_client = self.make_client(None);
        network::ensure_region_subscribed(&home_client, compartment_ocid, region).await?;

        let client = self.make_client(Some(region));

        let (vcn_id, route_table_id, ipv6_prefix) = {
            let (existing_vcn_id, existing_rt_id, raw_prefix) =
                match network::get_vcn_by_name(&client, compartment_ocid).await? {
                    Some(existing) => {
                        info!("Existing VCN found in {}, skipping VCN creation.", region);
                        existing
                    }
                    None => {
                        info!("No VCN in {}, creating...", region);
                        let ids = network::create_vcn(&client, compartment_ocid).await?;
                        info!("VCN created in {}.", region);
                        ids
                    }
                };

            if raw_prefix.is_empty() {
                let upgraded =
                    network::ensure_vcn_ipv6(&client, &existing_vcn_id, &raw_prefix).await;
                if !upgraded.is_empty() {
                    info!("IPv6 CIDR added to VCN in {}: {}", region, upgraded);
                    (existing_vcn_id, existing_rt_id, upgraded)
                } else {
                    info!(
                        "IPv6 upgrade failed for VCN in {}. Tearing down and recreating with IPv6...",
                        region
                    );
                    network::teardown_vcn_resources(
                        &client,
                        compartment_ocid,
                        &existing_vcn_id,
                        &existing_rt_id,
                    )
                    .await?;
                    let ids = network::create_vcn(&client, compartment_ocid).await?;
                    info!("VCN recreated with IPv6 in {}.", region);
                    ids
                }
            } else {
                (existing_vcn_id, existing_rt_id, raw_prefix)
            }
        };

        let igw_id =
            network::get_or_create_internet_gateway(&client, compartment_ocid, &vcn_id).await?;
        network::add_default_route_to_table(&client, &route_table_id, &igw_id, &ipv6_prefix)
            .await?;
        info!("IGW + default route ensured in {}.", region);

        let security_list_id =
            network::get_or_create_security_list(&client, compartment_ocid, &vcn_id, &ipv6_prefix)
                .await?;

        let subnet_id =
            match network::get_subnet_by_name(&client, compartment_ocid, &vcn_id).await? {
                Some((existing_id, _)) => {
                    info!(
                        "Subnet already exists in {}, ensuring security list.",
                        region
                    );
                    existing_id
                }
                None => {
                    let id = network::create_subnet(
                        &client,
                        compartment_ocid,
                        &vcn_id,
                        &security_list_id,
                        &route_table_id,
                        &ipv6_prefix,
                    )
                    .await?;
                    info!("Created subnet in {}", region);
                    id
                }
            };

        network::ensure_subnet_security_list(&client, &subnet_id, &security_list_id).await?;
        info!("Security list attached to subnet in {}.", region);

        Ok(())
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let client = self.make_client(Some(params.region));
        let compartment_ocid = self.get_compartment_ocid();

        let (vcn_id, _, _) = network::get_vcn_by_name(&client, compartment_ocid)
            .await?
            .ok_or_else(|| NetworkProvisioningError::VpcNotFound {
                vpc_name: "byocvpn-vcn".to_string(),
            })?;

        let (subnet_ocid, subnet_has_ipv6) =
            network::get_subnet_by_name(&client, compartment_ocid, &vcn_id)
                .await?
                .ok_or_else(|| NetworkProvisioningError::SubnetMissingIdentifier {})?;

        let image_ocid = network::get_ubuntu_image(&client, compartment_ocid).await?;

        instance::spawn_instance(
            &client,
            compartment_ocid,
            &subnet_ocid,
            subnet_has_ipv6,
            &image_ocid,
            params.region,
            params,
        )
        .await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        let client = self.make_client(Some(params.region));
        instance::terminate_instance(&client, params.instance_id).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        if let Some(region_name) = region {
            let client = self.make_client(Some(region_name));
            return instance::list_instances(&client, self.get_compartment_ocid(), region_name)
                .await;
        }

        let client = self.make_client(None);
        let region_list = network::list_regions(&client, self.get_compartment_ocid()).await?;
        let regions: Vec<Region> = region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect();
        let results = futures::future::join_all(regions.iter().map(|r| async move {
            let client = self.make_client(Some(&r.name));
            match instance::list_instances(&client, self.get_compartment_ocid(), &r.name).await {
                Ok(instances) => instances,
                Err(e) => {
                    error!("Skipping OCI region {}: {}", r.name, e);
                    vec![]
                }
            }
        }))
        .await;
        Ok(results.into_iter().flatten().collect())
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let client = self.make_client(None);
        let region_list = network::list_all_regions(&client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
