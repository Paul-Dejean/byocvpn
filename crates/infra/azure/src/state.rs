use byocvpn_core::cloud_provider::InstanceState;
use strum::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "PascalCase")]
pub enum AzureProvisioningState {
    Creating,
    Updating,
    Succeeded,
    Deleting,
    Deleted,
    Failed,
    Canceled,
    Unknown,
}

impl AzureProvisioningState {
    pub(super) fn from_state_name(state_name: &str) -> Self {
        state_name.parse().unwrap_or(Self::Unknown)
    }
}

impl From<AzureProvisioningState> for InstanceState {
    fn from(state: AzureProvisioningState) -> Self {
        match state {
            AzureProvisioningState::Creating => Self::Unknown,
            AzureProvisioningState::Updating => Self::Unknown,
            AzureProvisioningState::Succeeded => Self::Running,
            AzureProvisioningState::Deleting => Self::Unknown,
            AzureProvisioningState::Deleted => Self::Unknown,
            AzureProvisioningState::Failed => Self::Error,
            AzureProvisioningState::Canceled => Self::Error,
            AzureProvisioningState::Unknown => Self::Unknown,
        }
    }
}
