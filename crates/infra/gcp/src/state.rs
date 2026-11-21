use byocvpn_core::cloud_provider::InstanceState;
use google_cloud_compute_v1::model::instance::Status as GcpInstanceStatus;
use serde::Deserialize;
use strum::{Display, EnumString};

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

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum GcpServiceState {
    Enabled,
    Disabled,
    #[strum(default)]
    Other(String),
}

impl From<String> for GcpServiceState {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}
