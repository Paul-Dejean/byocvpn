use byocvpn_core::cloud_provider::InstanceState;
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Ec2InstanceState {
    Pending,
    Running,
    ShuttingDown,
    Terminated,
    Stopping,
    Stopped,
    Unknown,
}

impl Ec2InstanceState {
    pub(super) fn from_state_name(state_name: &str) -> Self {
        state_name.parse().unwrap_or(Self::Unknown)
    }
}

impl From<Ec2InstanceState> for InstanceState {
    fn from(state: Ec2InstanceState) -> Self {
        match state {
            Ec2InstanceState::Running => Self::Running,
            Ec2InstanceState::ShuttingDown | Ec2InstanceState::Stopping => Self::Stopping,
            Ec2InstanceState::Stopped => Self::Stopped,
            Ec2InstanceState::Pending
            | Ec2InstanceState::Terminated
            | Ec2InstanceState::Unknown => Self::Unknown,
        }
    }
}
