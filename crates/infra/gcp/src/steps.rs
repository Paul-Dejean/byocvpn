use byocvpn_core::{
    cloud_provider::{CloudProviderName, JobStep},
    error::Result,
    jobs::build_unknown_step_error,
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum GcpStepId {
    SetupApi,
    SetupVpc,
    SetupFirewall,
    RegionSubnet,
    SetupNetwork,
    Launch,
    WireguardReady,
}

impl GcpStepId {
    pub const fn get_label(self) -> &'static str {
        match self {
            Self::SetupApi => "Enabling Compute Engine API",
            Self::SetupVpc => "Creating VPC network",
            Self::SetupFirewall => "Creating firewall rules",
            Self::RegionSubnet => "Creating regional subnet",
            Self::SetupNetwork => "Verifying network infrastructure",
            Self::Launch => "Launching Compute Engine instance",
            Self::WireguardReady => "Waiting for WireGuard to start",
        }
    }

    fn build_job_step(self) -> JobStep {
        JobStep::new(self, self.get_label())
    }
}

pub fn parse_step_id(step_id: &str) -> Result<GcpStepId> {
    step_id
        .parse()
        .map_err(|_| build_unknown_step_error(CloudProviderName::Gcp, step_id))
}

pub fn build_provision_account_steps() -> Vec<JobStep> {
    [
        GcpStepId::SetupApi,
        GcpStepId::SetupVpc,
        GcpStepId::SetupFirewall,
    ]
    .into_iter()
    .map(GcpStepId::build_job_step)
    .collect()
}

pub fn build_enable_region_steps() -> Vec<JobStep> {
    vec![GcpStepId::RegionSubnet.build_job_step()]
}

pub fn build_spawn_steps() -> Vec<JobStep> {
    [
        GcpStepId::SetupNetwork,
        GcpStepId::Launch,
        GcpStepId::WireguardReady,
    ]
    .into_iter()
    .map(GcpStepId::build_job_step)
    .collect()
}
