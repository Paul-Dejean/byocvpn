use byocvpn_core::cloud_provider::InstanceState;
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
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
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OciResourceLifecycleState {
    Provisioning,
    Available,
    Updating,
    Terminating,
    Terminated,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OciRegionSubscriptionStatus {
    Ready,
    InProgress,
    #[strum(default)]
    Other(String),
}

impl From<String> for OciLifecycleState {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<String> for OciResourceLifecycleState {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<String> for OciRegionSubscriptionStatus {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<OciLifecycleState> for InstanceState {
    fn from(state: OciLifecycleState) -> Self {
        match state {
            OciLifecycleState::Running => Self::Running,
            OciLifecycleState::Stopping => Self::Stopping,
            OciLifecycleState::Stopped => Self::Stopped,
            OciLifecycleState::Moving
            | OciLifecycleState::Provisioning
            | OciLifecycleState::Starting
            | OciLifecycleState::CreatingImage
            | OciLifecycleState::Terminating
            | OciLifecycleState::Terminated
            | OciLifecycleState::Other(_) => Self::Unknown,
        }
    }
}
