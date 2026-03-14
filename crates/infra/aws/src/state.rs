use byocvpn_core::cloud_provider::InstanceState;

pub enum Ec2InstanceState {
    Pending,
    Running,
    ShuttingDown,
    Terminated,
    Stopping,
    Stopped,
    Unknown,
}

impl From<&str> for Ec2InstanceState {
    fn from(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "running" => Self::Running,
            "shutting-down" => Self::ShuttingDown,
            "terminated" => Self::Terminated,
            "stopping" => Self::Stopping,
            "stopped" => Self::Stopped,
            _ => Self::Unknown,
        }
    }
}

impl From<Ec2InstanceState> for InstanceState {
    fn from(state: Ec2InstanceState) -> Self {
        match state {
            Ec2InstanceState::Pending => InstanceState::Creating,
            Ec2InstanceState::Running => InstanceState::Running,
            Ec2InstanceState::ShuttingDown => InstanceState::Deleting,
            Ec2InstanceState::Terminated => InstanceState::Deleted,
            Ec2InstanceState::Stopping => InstanceState::Stopping,
            Ec2InstanceState::Stopped => InstanceState::Stopped,
            Ec2InstanceState::Unknown => InstanceState::Unknown,
        }
    }
}
