use byocvpn_core::cloud_provider::InstanceState;

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

impl From<&str> for AzureProvisioningState {
    fn from(s: &str) -> Self {
        match s {
            "Creating" => Self::Creating,
            "Updating" => Self::Updating,
            "Succeeded" => Self::Succeeded,
            "Deleting" => Self::Deleting,
            "Deleted" => Self::Deleted,
            "Failed" => Self::Failed,
            "Canceled" => Self::Canceled,
            _ => Self::Unknown,
        }
    }
}

impl From<AzureProvisioningState> for InstanceState {
    fn from(state: AzureProvisioningState) -> Self {
        match state {
            AzureProvisioningState::Creating => InstanceState::Creating,
            AzureProvisioningState::Updating => InstanceState::Creating,
            AzureProvisioningState::Succeeded => InstanceState::Running,
            AzureProvisioningState::Deleting => InstanceState::Deleting,
            AzureProvisioningState::Deleted => InstanceState::Deleted,
            AzureProvisioningState::Failed => InstanceState::Unknown,
            AzureProvisioningState::Canceled => InstanceState::Unknown,
            AzureProvisioningState::Unknown => InstanceState::Unknown,
        }
    }
}
