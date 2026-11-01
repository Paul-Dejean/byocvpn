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

use crate::{auth::OciCredentials, client::OciClient, instance, network, pricing};
use log::{debug, error, info};
use strum::{Display, EnumString};

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
    pub const fn new(config: OracleProviderConfig) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OracleSpawnStepId {
    SetupVcn,
    SetupIgw,
    RegionSubscribe,
    RegionVcn,
    RegionIgw,
    RegionSecurityList,
    RegionSubnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

#[async_trait]
impl CloudProvider for OracleProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Oracle
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![
            JobStep {
                id: OracleSpawnStepId::SetupNetwork.to_string(),
                label: "Verifying network infrastructure".into(),
            },
            JobStep {
                id: OracleSpawnStepId::Launch.to_string(),
                label: "Launching Ampere A1 instance".into(),
            },
            JobStep {
                id: OracleSpawnStepId::WireguardReady.to_string(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let step = step_id
            .parse::<OracleSpawnStepId>()
            .map_err(|_| build_unknown_step_error(CloudProviderName::Oracle, step_id))?;
        match step {
            OracleSpawnStepId::SetupNetwork => {
                self.ensure_account_network().await?;
                self.ensure_region_network(region).await
            }
            OracleSpawnStepId::SetupVcn => {
                let client = self.make_client(None);
                let compartment = self.get_compartment_ocid();
                network::ensure_vcn(&client, compartment).await?;
                Ok(())
            }
            OracleSpawnStepId::SetupIgw => {
                let client = self.make_client(None);
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::ensure_vcn(&client, compartment).await?;
                let igw_id =
                    network::ensure_internet_gateway(&client, compartment, &vcn_id).await?;
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
                network::ensure_vcn(&client, compartment).await?;
                Ok(())
            }
            OracleSpawnStepId::RegionIgw => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::ensure_vcn(&client, compartment).await?;
                let igw_id =
                    network::ensure_internet_gateway(&client, compartment, &vcn_id).await?;
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
                let (vcn_id, _, ipv6_prefix) = network::ensure_vcn(&client, compartment).await?;
                network::ensure_security_list(&client, compartment, &vcn_id, &ipv6_prefix).await?;
                Ok(())
            }
            OracleSpawnStepId::RegionSubnet => {
                let client = self.make_client(Some(region));
                let compartment = self.get_compartment_ocid();
                let (vcn_id, route_table_id, ipv6_prefix) =
                    network::ensure_vcn(&client, compartment).await?;
                let security_list_id =
                    network::ensure_security_list(&client, compartment, &vcn_id, &ipv6_prefix)
                        .await?;
                network::ensure_subnet(
                    &client,
                    compartment,
                    &vcn_id,
                    &security_list_id,
                    &route_table_id,
                    &ipv6_prefix,
                )
                .await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        Ok(Vec::new())
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        vec![
            JobStep {
                id: OracleSpawnStepId::SetupVcn.to_string(),
                label: "Creating home VCN".into(),
            },
            JobStep {
                id: OracleSpawnStepId::SetupIgw.to_string(),
                label: "Creating internet gateway".into(),
            },
        ]
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        self.run_spawn_step(step_id, "").await
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![
            JobStep {
                id: OracleSpawnStepId::RegionSubscribe.to_string(),
                label: "Subscribing to region \u{2014} this may take several minutes".into(),
            },
            JobStep {
                id: OracleSpawnStepId::RegionVcn.to_string(),
                label: "Creating regional VCN".into(),
            },
            JobStep {
                id: OracleSpawnStepId::RegionIgw.to_string(),
                label: "Creating regional internet gateway".into(),
            },
            JobStep {
                id: OracleSpawnStepId::RegionSecurityList.to_string(),
                label: "Creating security list".into(),
            },
            JobStep {
                id: OracleSpawnStepId::RegionSubnet.to_string(),
                label: "Creating subnet".into(),
            },
        ]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let client = self.make_client(Some(params.region));
        let compartment_ocid = self.get_compartment_ocid();

        let (vcn_id, route_table_id, ipv6_prefix) =
            network::ensure_vcn(&client, compartment_ocid).await?;
        debug!("Resolved VCN {} for spawn in {}", vcn_id, params.region);

        let security_list_id =
            network::ensure_security_list(&client, compartment_ocid, &vcn_id, &ipv6_prefix).await?;

        let subnet_ocid = network::ensure_subnet(
            &client,
            compartment_ocid,
            &vcn_id,
            &security_list_id,
            &route_table_id,
            &ipv6_prefix,
        )
        .await?;
        let subnet_has_ipv6 = !ipv6_prefix.is_empty();
        debug!(
            "Resolved subnet {} (IPv6: {}) in {}",
            subnet_ocid, subnet_has_ipv6, params.region
        );

        let image_ocid = network::get_ubuntu_image(&client, compartment_ocid).await?;
        debug!("Resolved Ubuntu image {} in {}", image_ocid, params.region);

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
        let results = futures::future::join_all(regions.iter().map(|region| async move {
            let client = self.make_client(Some(&region.name));
            match instance::list_instances(&client, self.get_compartment_ocid(), &region.name).await
            {
                Ok(instances) => instances,
                Err(error) => {
                    error!("Skipping OCI region {}: {}", region.name, error);
                    vec![]
                }
            }
        }))
        .await;
        Ok(results.into_iter().flatten().collect())
    }

    fn get_pricing(&self, instance_type: &str) -> Option<PricingInfo> {
        pricing::get_pricing(instance_type)
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

impl OracleProvider {
    async fn ensure_account_network(&self) -> Result<()> {
        self.run_spawn_step(&OracleSpawnStepId::SetupVcn.to_string(), "")
            .await?;
        self.run_spawn_step(&OracleSpawnStepId::SetupIgw.to_string(), "")
            .await?;
        Ok(())
    }

    async fn ensure_region_network(&self, region: &str) -> Result<()> {
        let compartment_ocid = self.get_compartment_ocid();

        let home_client = self.make_client(None);
        network::ensure_region_subscribed(&home_client, compartment_ocid, region).await?;

        let client = self.make_client(Some(region));

        let (vcn_id, route_table_id, ipv6_prefix) =
            network::ensure_vcn(&client, compartment_ocid).await?;

        let igw_id = network::ensure_internet_gateway(&client, compartment_ocid, &vcn_id).await?;
        network::add_default_route_to_table(&client, &route_table_id, &igw_id, &ipv6_prefix)
            .await?;
        info!("IGW + default route ensured in {region}.");

        let security_list_id =
            network::ensure_security_list(&client, compartment_ocid, &vcn_id, &ipv6_prefix).await?;

        network::ensure_subnet(
            &client,
            compartment_ocid,
            &vcn_id,
            &security_list_id,
            &route_table_id,
            &ipv6_prefix,
        )
        .await?;
        info!("Security list attached to subnet in {region}.");

        Ok(())
    }
}
