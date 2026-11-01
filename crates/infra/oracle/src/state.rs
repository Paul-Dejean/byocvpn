use byocvpn_core::cloud_provider::InstanceState;
use strum::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
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

impl OciLifecycleState {
    pub(super) fn from_state_name(state_name: &str) -> Self {
        state_name.parse().unwrap_or(Self::Unknown)
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
