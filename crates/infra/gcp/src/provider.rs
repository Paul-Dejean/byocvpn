use std::collections::HashSet;

use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, JobStep, PermissionStatus, PricingInfo,
        SpawnInstanceParams, TerminateInstanceParams,
    },
    commands::setup::Region,
    error::Result,
};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    auth::parse_credentials_from_service_account_json, client::GcpClient, instance, network,
    pricing,
};

const REQUIRED_PERMISSIONS: &[&str] = &[
    "compute.disks.create",
    "compute.disks.delete",
    "compute.firewalls.create",
    "compute.firewalls.delete",
    "compute.firewalls.get",
    "compute.firewalls.update",
    "compute.globalOperations.get",
    "compute.images.get",
    "compute.instances.create",
    "compute.instances.delete",
    "compute.instances.get",
    "compute.instances.list",
    "compute.instances.setLabels",
    "compute.instances.setMetadata",
    "compute.instances.setTags",
    "compute.networks.create",
    "compute.networks.delete",
    "compute.networks.get",
    "compute.networks.updatePolicy",
    "compute.regionOperations.get",
    "compute.regions.list",
    "compute.subnetworks.create",
    "compute.subnetworks.delete",
    "compute.subnetworks.get",
    "compute.subnetworks.list",
    "compute.subnetworks.update",
    "compute.subnetworks.use",
    "compute.subnetworks.useExternalIp",
    "compute.zoneOperations.get",
    "serviceusage.operations.get",
    "serviceusage.services.enable",
    "serviceusage.services.get",
];

#[derive(Serialize)]
struct TestIamPermissionsRequest {
    permissions: Vec<String>,
}

#[derive(Deserialize, Default)]
struct TestIamPermissionsResponse {
    #[serde(default)]
    permissions: Vec<String>,
}

pub struct GcpProviderConfig {
    pub service_account_json: String,
}

pub struct GcpProvider {
    client: GcpClient,
}

impl GcpProvider {
    pub fn new(config: GcpProviderConfig) -> Result<Self> {
        let (credentials, project_id) =
            parse_credentials_from_service_account_json(&config.service_account_json)?;
        let client = GcpClient::new(credentials, project_id);
        Ok(Self { client })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum GcpSpawnStepId {
    SetupApi,
    SetupVpc,
    SetupFirewall,
    RegionSubnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

#[async_trait]
impl CloudProvider for GcpProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Gcp
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![
            JobStep {
                id: GcpSpawnStepId::SetupNetwork.to_string(),
                label: "Verifying network infrastructure".into(),
            },
            JobStep {
                id: GcpSpawnStepId::Launch.to_string(),
                label: "Launching Compute Engine instance".into(),
            },
            JobStep {
                id: GcpSpawnStepId::WireguardReady.to_string(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<GcpSpawnStepId>() else {
            return Ok(());
        };
        match step {
            GcpSpawnStepId::SetupApi => {
                network::ensure_compute_api_enabled(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::SetupVpc => {
                network::ensure_vpc(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::SetupFirewall => {
                network::ensure_firewall_rules(&self.client).await?;
                Ok(())
            }
            GcpSpawnStepId::RegionSubnet => {
                network::ensure_subnet(&self.client, region).await?;
                Ok(())
            }
            GcpSpawnStepId::SetupNetwork => {
                self.ensure_account_network().await?;
                self.ensure_region_network(region).await
            }
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Vec<PermissionStatus>> {
        network::ensure_cloud_resource_manager_api_enabled(&self.client).await?;

        let url = format!(
            "https://cloudresourcemanager.googleapis.com/v1/projects/{}:testIamPermissions",
            self.client.project_id
        );
        let request = TestIamPermissionsRequest {
            permissions: REQUIRED_PERMISSIONS
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        };

        let response: TestIamPermissionsResponse = self.client.post(&url, &request).await?;
        let granted_permissions: HashSet<&str> = response
            .permissions
            .iter()
            .map(std::string::String::as_str)
            .collect();

        let permissions: Vec<PermissionStatus> = REQUIRED_PERMISSIONS
            .iter()
            .map(|permission| {
                let granted = granted_permissions.contains(permission);
                if granted {
                    info!("permission check {permission}: authorized");
                } else {
                    warn!("permission check {permission}: denied");
                }
                PermissionStatus {
                    permission: permission.to_string(),
                    granted,
                }
            })
            .collect();

        Ok(permissions)
    }

    fn get_provision_account_steps(&self) -> Vec<JobStep> {
        vec![
            JobStep {
                id: GcpSpawnStepId::SetupApi.to_string(),
                label: "Enabling Compute Engine API".into(),
            },
            JobStep {
                id: GcpSpawnStepId::SetupVpc.to_string(),
                label: "Creating VPC network".into(),
            },
            JobStep {
                id: GcpSpawnStepId::SetupFirewall.to_string(),
                label: "Creating firewall rules".into(),
            },
        ]
    }

    async fn run_provision_account_step(&self, step_id: &str) -> Result<()> {
        self.run_spawn_step(step_id, "").await
    }

    fn get_enable_region_steps(&self, _region: &str) -> Vec<JobStep> {
        vec![JobStep {
            id: GcpSpawnStepId::RegionSubnet.to_string(),
            label: "Creating regional subnet".into(),
        }]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        let subnet_self_link = network::ensure_subnet(&self.client, params.region)
            .await
            .map_err(|error| error.with_context("subnet creation failed"))?;
        debug!(
            "Resolved GCP subnet {} for spawn in {}",
            subnet_self_link, params.region
        );

        let image_self_link = network::get_ubuntu_image_self_link(&self.client).await?;
        debug!("Resolved GCP Ubuntu image {image_self_link}");

        instance::spawn_instance(
            &self.client,
            &subnet_self_link,
            &image_self_link,
            params.region,
            params,
        )
        .await
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

    fn get_pricing(&self, instance_type: &str) -> Option<PricingInfo> {
        pricing::get_pricing(instance_type)
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        network::ensure_compute_api_enabled(&self.client).await?;
        let region_list = network::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}

impl GcpProvider {
    async fn ensure_account_network(&self) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::ensure_vpc(&self.client).await?;
        network::ensure_firewall_rules(&self.client).await?;
        info!("GCP setup complete (VPC + firewall).");
        Ok(())
    }

    async fn ensure_region_network(&self, region: &str) -> Result<()> {
        network::ensure_compute_api_enabled(&self.client).await?;
        network::ensure_vpc(&self.client).await?;
        network::ensure_firewall_rules(&self.client).await?;
        network::ensure_subnet(&self.client, region).await?;
        info!("GCP region {region} enabled.");
        Ok(())
    }
}
