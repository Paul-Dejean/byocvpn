use byocvpn_core::cloud_provider::InstanceState;
use strum::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum GcpInstanceStatus {
    Provisioning,
    Staging,
    Running,
    Stopping,
    Suspending,
    Suspended,
    Repairing,
    Terminated,
    Unknown,
}

impl GcpInstanceStatus {
    pub(super) fn from_state_name(state_name: &str) -> Self {
        state_name.parse().unwrap_or(Self::Unknown)
    }
}

impl From<GcpInstanceStatus> for InstanceState {
    fn from(status: GcpInstanceStatus) -> Self {
        match status {
            GcpInstanceStatus::Provisioning => Self::Unknown,
            GcpInstanceStatus::Staging => Self::Unknown,
            GcpInstanceStatus::Running => Self::Running,
            GcpInstanceStatus::Stopping => Self::Stopping,
            GcpInstanceStatus::Suspending => Self::Stopping,
            GcpInstanceStatus::Suspended => Self::Stopped,
            GcpInstanceStatus::Repairing => Self::Error,
            GcpInstanceStatus::Terminated => Self::Unknown,
            GcpInstanceStatus::Unknown => Self::Unknown,
        }
    }
}
