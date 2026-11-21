use serde_json::Error as JsonError;
use thiserror::Error;

use crate::cloud_provider::CloudProviderName;

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("lock {lock_name} is poisoned")]
    MutexPoisoned { lock_name: &'static str },

    #[error("{provider} has no job step with id {step_id}")]
    UnknownJobStep {
        provider: CloudProviderName,
        step_id: String,
    },

    #[error("JSON serialization failed")]
    Json(#[from] JsonError),
}

impl InternalError {
    pub fn unknown_step(provider: CloudProviderName, step_id: &str) -> Self {
        Self::UnknownJobStep {
            provider,
            step_id: step_id.to_string(),
        }
    }
}
