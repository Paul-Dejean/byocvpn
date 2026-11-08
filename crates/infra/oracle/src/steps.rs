use byocvpn_core::{
    cloud_provider::{CloudProviderName, JobStep},
    error::Result,
    jobs::build_unknown_step_error,
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OracleStepId {
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

impl OracleStepId {
    pub const fn get_label(self) -> &'static str {
        match self {
            Self::SetupVcn => "Creating home VCN",
            Self::SetupIgw => "Creating internet gateway",
            Self::RegionSubscribe => "Subscribing to region \u{2014} this may take several minutes",
            Self::RegionVcn => "Creating regional VCN",
            Self::RegionIgw => "Creating regional internet gateway",
            Self::RegionSecurityList => "Creating security list",
            Self::RegionSubnet => "Creating subnet",
            Self::SetupNetwork => "Verifying network infrastructure",
            Self::Launch => "Launching Ampere A1 instance",
            Self::WireguardReady => "Waiting for WireGuard to start",
        }
    }

    fn build_job_step(self) -> JobStep {
        JobStep::new(self, self.get_label())
    }
}

pub fn parse_step_id(step_id: &str) -> Result<OracleStepId> {
    step_id
        .parse()
        .map_err(|_| build_unknown_step_error(CloudProviderName::Oracle, step_id))
}

pub fn build_provision_account_steps() -> Vec<JobStep> {
    [OracleStepId::SetupVcn, OracleStepId::SetupIgw]
        .into_iter()
        .map(OracleStepId::build_job_step)
        .collect()
}

pub fn build_enable_region_steps() -> Vec<JobStep> {
    [
        OracleStepId::RegionSubscribe,
        OracleStepId::RegionVcn,
        OracleStepId::RegionIgw,
        OracleStepId::RegionSecurityList,
        OracleStepId::RegionSubnet,
    ]
    .into_iter()
    .map(OracleStepId::build_job_step)
    .collect()
}

pub fn build_spawn_steps() -> Vec<JobStep> {
    [
        OracleStepId::SetupNetwork,
        OracleStepId::Launch,
        OracleStepId::WireguardReady,
    ]
    .into_iter()
    .map(OracleStepId::build_job_step)
    .collect()
}
