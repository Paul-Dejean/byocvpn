use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] byocvpn_core::error::Error),

    #[error("Already connected to {instance_id}. Run `disconnect` first.")]
    AlreadyConnected { instance_id: String },

    #[error("Instance {instance_id} was not found")]
    InstanceNotFound { instance_id: String },

    #[error("No cloud provider credentials found. Add your credentials in the desktop app first.")]
    NoProviderConfigured,

    #[error("No servers found")]
    NoInstancesFound,

    #[error("No region selected")]
    RegionRequired,

    #[error("Prompt failed: {0}")]
    Prompt(#[from] dialoguer::Error),
}

pub type CliResult<T> = std::result::Result<T, CliError>;
