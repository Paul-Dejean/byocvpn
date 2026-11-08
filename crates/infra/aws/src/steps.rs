use byocvpn_core::{
    cloud_provider::{CloudProviderName, JobStep},
    error::Result,
    jobs::build_unknown_step_error,
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AwsStepId {
    SetupVpc,
    SetupIgw,
    RegionSubnets,
    RegionSecurityGroup,
    SetupNetwork,
    Launch,
    WireguardReady,
}

impl AwsStepId {
    pub const fn get_label(self) -> &'static str {
        match self {
            Self::SetupVpc => "Creating VPC",
            Self::SetupIgw => "Creating internet gateway",
            Self::RegionSubnets => "Creating subnets",
            Self::RegionSecurityGroup => "Configuring security group",
            Self::SetupNetwork => "Verifying network infrastructure",
            Self::Launch => "Launching EC2 instance",
            Self::WireguardReady => "Waiting for WireGuard to start",
        }
    }

    fn build_job_step(self) -> JobStep {
        JobStep::new(self, self.get_label())
    }
}

pub fn parse_step_id(step_id: &str) -> Result<AwsStepId> {
    step_id
        .parse()
        .map_err(|_| build_unknown_step_error(CloudProviderName::Aws, step_id))
}

pub const fn build_provision_account_steps() -> Vec<JobStep> {
    Vec::new()
}

pub fn build_enable_region_steps() -> Vec<JobStep> {
    [
        AwsStepId::SetupVpc,
        AwsStepId::SetupIgw,
        AwsStepId::RegionSubnets,
        AwsStepId::RegionSecurityGroup,
    ]
    .into_iter()
    .map(AwsStepId::build_job_step)
    .collect()
}

pub fn build_spawn_steps() -> Vec<JobStep> {
    [
        AwsStepId::SetupNetwork,
        AwsStepId::Launch,
        AwsStepId::WireguardReady,
    ]
    .into_iter()
    .map(AwsStepId::build_job_step)
    .collect()
}
