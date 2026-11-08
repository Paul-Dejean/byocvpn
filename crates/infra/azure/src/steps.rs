use byocvpn_core::{
    cloud_provider::{CloudProviderName, JobStep},
    error::Result,
    jobs::build_unknown_step_error,
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AzureStepId {
    RegionResourceGroup,
    RegionVnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

impl AzureStepId {
    pub const fn get_label(self) -> &'static str {
        match self {
            Self::RegionResourceGroup => "Creating resource group",
            Self::RegionVnet => "Creating VNet and subnet",
            Self::SetupNetwork => "Verifying network infrastructure",
            Self::Launch => "Launching virtual machine",
            Self::WireguardReady => "Waiting for WireGuard to start",
        }
    }

    fn build_job_step(self) -> JobStep {
        JobStep::new(self, self.get_label())
    }
}

pub fn parse_step_id(step_id: &str) -> Result<AzureStepId> {
    step_id
        .parse()
        .map_err(|_| build_unknown_step_error(CloudProviderName::Azure, step_id))
}

pub const fn build_provision_account_steps() -> Vec<JobStep> {
    Vec::new()
}

pub fn build_enable_region_steps() -> Vec<JobStep> {
    [AzureStepId::RegionResourceGroup, AzureStepId::RegionVnet]
        .into_iter()
        .map(AzureStepId::build_job_step)
        .collect()
}

pub fn build_spawn_steps() -> Vec<JobStep> {
    [
        AzureStepId::SetupNetwork,
        AzureStepId::Launch,
        AzureStepId::WireguardReady,
    ]
    .into_iter()
    .map(AzureStepId::build_job_step)
    .collect()
}
