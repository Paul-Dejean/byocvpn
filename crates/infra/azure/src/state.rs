use byocvpn_core::cloud_provider::InstanceState;
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
pub enum AzureProvisioningState {
    Creating,
    Updating,
    Succeeded,
    Deleting,
    Deleted,
    Failed,
    Canceled,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
pub enum AzureOperationStatus {
    Succeeded,
    Failed,
    Canceled,
    InProgress,
    Running,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize)]
#[serde(from = "String")]
pub enum AzureRegistrationState {
    Registered,
    Registering,
    NotRegistered,
    Unregistering,
    #[strum(default)]
    Other(String),
}

impl From<String> for AzureProvisioningState {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<String> for AzureOperationStatus {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<String> for AzureRegistrationState {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Other(value))
    }
}

impl From<AzureProvisioningState> for InstanceState {
    fn from(state: AzureProvisioningState) -> Self {
        match state {
            AzureProvisioningState::Succeeded => Self::Running,
            AzureProvisioningState::Failed | AzureProvisioningState::Canceled => Self::Error,
            AzureProvisioningState::Creating
            | AzureProvisioningState::Updating
            | AzureProvisioningState::Deleting
            | AzureProvisioningState::Deleted
            | AzureProvisioningState::Other(_) => Self::Unknown,
        }
    }
}
