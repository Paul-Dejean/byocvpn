use byocvpn_core::cloud_provider::InstanceState;

pub enum OciLifecycleState {
    Moving,
    Provisioning,
    Starting,
    Running,
    Stopping,
    Stopped,
    CreatingImage,
    Terminating,
    Terminated,
    Unknown,
}

impl From<&str> for OciLifecycleState {
    fn from(s: &str) -> Self {
        match s {
            "MOVING" => Self::Moving,
            "PROVISIONING" => Self::Provisioning,
            "STARTING" => Self::Starting,
            "RUNNING" => Self::Running,
            "STOPPING" => Self::Stopping,
            "STOPPED" => Self::Stopped,
            "CREATING_IMAGE" => Self::CreatingImage,
            "TERMINATING" => Self::Terminating,
            "TERMINATED" => Self::Terminated,
            _ => Self::Unknown,
        }
    }
}

impl From<OciLifecycleState> for InstanceState {
    fn from(state: OciLifecycleState) -> Self {
        match state {
            OciLifecycleState::Moving => InstanceState::Creating,
            OciLifecycleState::Provisioning => InstanceState::Creating,
            OciLifecycleState::Starting => InstanceState::Creating,
            OciLifecycleState::Running => InstanceState::Running,
            OciLifecycleState::Stopping => InstanceState::Stopping,
            OciLifecycleState::Stopped => InstanceState::Stopped,
            OciLifecycleState::CreatingImage => InstanceState::Unknown,
            OciLifecycleState::Terminating => InstanceState::Deleting,
            OciLifecycleState::Terminated => InstanceState::Deleted,
            OciLifecycleState::Unknown => InstanceState::Unknown,
        }
    }
}
