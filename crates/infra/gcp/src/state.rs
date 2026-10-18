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
