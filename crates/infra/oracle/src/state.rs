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
            OciLifecycleState::Moving => Self::Unknown,
            OciLifecycleState::Provisioning => Self::Unknown,
            OciLifecycleState::Starting => Self::Unknown,
            OciLifecycleState::Running => Self::Running,
            OciLifecycleState::Stopping => Self::Stopping,
            OciLifecycleState::Stopped => Self::Stopped,
            OciLifecycleState::CreatingImage => Self::Unknown,
            OciLifecycleState::Terminating => Self::Unknown,
            OciLifecycleState::Terminated => Self::Unknown,
            OciLifecycleState::Unknown => Self::Unknown,
        }
    }
}
