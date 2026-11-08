use byocvpn_core::cloud_provider::InstanceState;
use google_cloud_compute_v1::model::instance::Status as GcpInstanceStatus;

pub const fn map_instance_state(status: Option<&GcpInstanceStatus>) -> InstanceState {
    match status {
        Some(GcpInstanceStatus::Running) => InstanceState::Running,
        Some(
            GcpInstanceStatus::Stopping
            | GcpInstanceStatus::Suspending
            | GcpInstanceStatus::PendingStop,
        ) => InstanceState::Stopping,
        Some(GcpInstanceStatus::Suspended | GcpInstanceStatus::Stopped) => InstanceState::Stopped,
        Some(GcpInstanceStatus::Repairing) => InstanceState::Error,
        _ => InstanceState::Unknown,
    }
}
