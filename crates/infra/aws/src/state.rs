use byocvpn_core::cloud_provider::InstanceState;
use strum::EnumString;

#[derive(EnumString)]
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
            Ec2InstanceState::Pending => Self::Unknown,
            Ec2InstanceState::Running => Self::Running,
            Ec2InstanceState::ShuttingDown => Self::Stopping,
            Ec2InstanceState::Terminated => Self::Unknown,
            Ec2InstanceState::Stopping => Self::Stopping,
            Ec2InstanceState::Stopped => Self::Stopped,
            Ec2InstanceState::Unknown => Self::Unknown,
        }
    }
}
