use std::str::FromStr;

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

    fn from_str(step_id: &str) -> std::result::Result<Self, ()> {
        match step_id {
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
