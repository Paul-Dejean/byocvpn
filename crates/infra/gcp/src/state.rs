use byocvpn_core::cloud_provider::InstanceState;

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

impl From<&str> for GcpInstanceStatus {
    fn from(s: &str) -> Self {
        match s {
            "PROVISIONING" => Self::Provisioning,
            "STAGING" => Self::Staging,
            "RUNNING" => Self::Running,
            "STOPPING" => Self::Stopping,
            "SUSPENDING" => Self::Suspending,
            "SUSPENDED" => Self::Suspended,
            "REPAIRING" => Self::Repairing,
            "TERMINATED" => Self::Terminated,
            _ => Self::Unknown,
        }
    }
}

impl From<GcpInstanceStatus> for InstanceState {
    fn from(status: GcpInstanceStatus) -> Self {
        match status {
            GcpInstanceStatus::Provisioning => InstanceState::Creating,
            GcpInstanceStatus::Staging => InstanceState::Creating,
            GcpInstanceStatus::Running => InstanceState::Running,
            GcpInstanceStatus::Stopping => InstanceState::Stopping,
            GcpInstanceStatus::Suspending => InstanceState::Stopping,
            GcpInstanceStatus::Suspended => InstanceState::Stopped,
            GcpInstanceStatus::Repairing => InstanceState::Unknown,
            GcpInstanceStatus::Terminated => InstanceState::Deleted,
            GcpInstanceStatus::Unknown => InstanceState::Unknown,
        }
    }
}
