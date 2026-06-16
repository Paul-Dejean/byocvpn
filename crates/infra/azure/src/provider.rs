use std::str::FromStr;

use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, PermissionStatus, SpawnInstanceParams,
        SpawnStep, TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use log::*;
use serde::Deserialize;
use serde_json::Value;

use crate::{auth::create_credential, client::AzureClient, instance, network};

const REQUIRED_ACTIONS: &[&str] = &[
    "Microsoft.Compute/register/action",
    "Microsoft.Compute/virtualMachines/read",
    "Microsoft.Compute/virtualMachines/write",
    "Microsoft.Compute/virtualMachines/delete",
    "Microsoft.Network/register/action",
    "Microsoft.Network/networkInterfaces/join/action",
    "Microsoft.Network/networkInterfaces/read",
    "Microsoft.Network/networkInterfaces/write",
    "Microsoft.Network/networkInterfaces/delete",
    "Microsoft.Network/networkSecurityGroups/join/action",
    "Microsoft.Network/networkSecurityGroups/read",
    "Microsoft.Network/networkSecurityGroups/write",
    "Microsoft.Network/networkSecurityGroups/delete",
    "Microsoft.Network/publicIPAddresses/join/action",
    "Microsoft.Network/publicIPAddresses/read",
    "Microsoft.Network/publicIPAddresses/write",
    "Microsoft.Network/publicIPAddresses/delete",
    "Microsoft.Network/virtualNetworks/read",
    "Microsoft.Network/virtualNetworks/write",
    "Microsoft.Network/virtualNetworks/delete",
    "Microsoft.Network/virtualNetworks/subnets/join/action",
    "Microsoft.Network/virtualNetworks/subnets/read",
    "Microsoft.Network/virtualNetworks/subnets/write",
    "Microsoft.Network/virtualNetworks/subnets/delete",
    "Microsoft.Resources/subscriptions/locations/read",
    "Microsoft.Resources/subscriptions/providers/read",
    "Microsoft.Resources/subscriptions/resourceGroups/read",
    "Microsoft.Resources/subscriptions/resourceGroups/write",
    "Microsoft.Resources/subscriptions/resourceGroups/delete",
];

#[derive(Deserialize)]
struct PermissionsResponse {
    #[serde(default)]
    value: Vec<PermissionSet>,
}

#[derive(Deserialize, Default)]
struct PermissionSet {
    #[serde(default)]
    actions: Vec<String>,
    #[serde(default, rename = "notActions")]
    not_actions: Vec<String>,
}

fn wildcard_matches(pattern: &str, action: &str) -> bool {
    let pattern: Vec<char> = pattern.to_lowercase().chars().collect();
    let action: Vec<char> = action.to_lowercase().chars().collect();
    let mut pattern_index = 0;
    let mut action_index = 0;
    let mut star_pattern_index: Option<usize> = None;
    let mut star_action_index = 0;

    while action_index < action.len() {
        if pattern_index < pattern.len() && pattern[pattern_index] == action[action_index] {
            pattern_index += 1;
            action_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star_pattern_index = Some(pattern_index);
            star_action_index = action_index;
            pattern_index += 1;
        } else if let Some(star_index) = star_pattern_index {
            pattern_index = star_index + 1;
            star_action_index += 1;
            action_index = star_action_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn is_action_granted(action: &str, permission_sets: &[PermissionSet]) -> bool {
    permission_sets.iter().any(|permission_set| {
        let allowed = permission_set
            .actions
            .iter()
            .any(|pattern| wildcard_matches(pattern, action));
        let denied = permission_set
            .not_actions
            .iter()
            .any(|pattern| wildcard_matches(pattern, action));
        allowed && !denied
    })
}

pub struct AzureProviderConfig {
    pub subscription_id: String,
    pub tenant_id: String,
    pub application_id: String,
    pub secret_value: String,
}

pub struct AzureProvider {
    client: AzureClient,
}

impl AzureProvider {
    pub fn new(config: AzureProviderConfig) -> Result<Self> {
        let credential = create_credential(
            &config.tenant_id,
            &config.application_id,
            &config.secret_value,
        )?;
        let client = AzureClient::new(credential, config.subscription_id);
        Ok(Self { client })
    }
}

pub enum AzureSpawnStepId {
    RegionResourceGroup,
    RegionVnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

impl AzureSpawnStepId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegionResourceGroup => "region_resource_group",
            Self::RegionVnet => "region_vnet",
            Self::SetupNetwork => "setup_network",
            Self::Launch => "launch",
            Self::WireguardReady => "wireguard_ready",
        }
    }
}

impl FromStr for AzureSpawnStepId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "region_resource_group" => Ok(Self::RegionResourceGroup),
            "region_vnet" => Ok(Self::RegionVnet),
            "setup_network" => Ok(Self::SetupNetwork),
            "launch" => Ok(Self::Launch),
            "wireguard_ready" => Ok(Self::WireguardReady),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl CloudProvider for AzureProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    fn get_spawn_steps(&self, _region: &str) -> Vec<SpawnStep> {
        vec![
            SpawnStep {
                id: AzureSpawnStepId::SetupNetwork.as_str().into(),
                label: "Verifying network infrastructure".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::Launch.as_str().into(),
                label: "Launching virtual machine".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::WireguardReady.as_str().into(),
                label: "Waiting for WireGuard to start".into(),
            },
        ]
    }

    async fn run_spawn_step(&self, step_id: &str, region: &str) -> Result<()> {
        let Ok(step) = step_id.parse::<AzureSpawnStepId>() else {
            return Ok(());
        };
        match step {
            AzureSpawnStepId::RegionResourceGroup => {
                network::ensure_resource_group(&self.client, region).await
            }
            AzureSpawnStepId::RegionVnet => {
                let nsg_id = network::ensure_nsg(&self.client, region).await?;
                network::ensure_vnet(&self.client, region).await?;
                network::ensure_subnet(&self.client, region, &nsg_id).await?;
                Ok(())
            }
            AzureSpawnStepId::SetupNetwork => self.enable_region(region).await,
            _ => Ok(()),
        }
    }

    async fn verify_permissions(&self) -> Result<Value> {
        let path = self
            .client
            .build_subscription_path("/providers/Microsoft.Authorization/permissions");
        let url = self.client.build_arm_url(&path, "2018-07-01");

        let response: PermissionsResponse = self.client.get(&url).await?;

        let permissions: Vec<PermissionStatus> = REQUIRED_ACTIONS
            .iter()
            .map(|action| {
                let granted = is_action_granted(action, &response.value);
                if granted {
                    info!("permission check {action}: authorized");
                } else {
                    warn!("permission check {action}: denied");
                }
                PermissionStatus {
                    permission: action.to_string(),
                    granted,
                }
            })
            .collect();

        let value = serde_json::to_value(&permissions).map_err(|error| {
            NetworkProvisioningError::NetworkQueryFailed {
                reason: error.to_string(),
            }
        })?;
        Ok(value)
    }

    async fn setup(&self) -> Result<()> {
        info!("[Azure] setup() — no global resources required.");
        Ok(())
    }

    async fn enable_region(&self, region: &str) -> Result<()> {
        network::ensure_region_networking(&self.client, region).await?;
        info!("[Azure] Region '{}' enabled.", region);
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
                id: AzureSpawnStepId::RegionResourceGroup.as_str().into(),
                label: "Creating resource group".into(),
            },
            SpawnStep {
                id: AzureSpawnStepId::RegionVnet.as_str().into(),
                label: "Creating VNet and subnet".into(),
            },
        ]
    }

    async fn run_enable_region_step(&self, step_id: &str, region: &str) -> Result<()> {
        self.run_spawn_step(step_id, region).await
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        instance::spawn_instance(&self.client, params.region, params).await
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

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let region_list = network::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
