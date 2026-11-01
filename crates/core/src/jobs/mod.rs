pub mod enable_region;
pub mod provision_account;
pub mod region_preparation;
pub mod spawn;

use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error};

use crate::{
    cloud_provider::{CloudProviderName, JobStep, JobStepStatus},
    error::{Error, ErrorPayload, Result, SystemError},
};

pub trait JobObserver: Send + Sync {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, error: Option<ErrorPayload>);
}

#[async_trait]
pub trait JobSteps: Send {
    async fn run_step(&mut self, step: &JobStep) -> Result<()>;

    fn on_step_completed(&mut self, _step: &JobStep) {}
}

pub async fn run_job_steps(
    steps: &[JobStep],
    job_steps: &mut dyn JobSteps,
    observer: &dyn JobObserver,
) -> Result<()> {
    for step in steps {
        debug!("Running job step: {}", step.id);
        observer.on_step_changed(step, JobStepStatus::Running, None);

        if let Err(step_error) = job_steps.run_step(step).await {
            error!("Job step {} failed: {step_error}", step.id);
            observer.on_step_changed(step, JobStepStatus::Failed, Some(step_error.to_payload()));
            return Err(step_error);
        }

        observer.on_step_changed(step, JobStepStatus::Completed, None);
        job_steps.on_step_completed(step);
    }
    Ok(())
}

pub fn build_job_id(provider: &CloudProviderName, region: Option<&str>) -> String {
    let timestamp = Utc::now().timestamp_millis();
    match region {
        Some(region) => format!("{provider}-{region}-{timestamp}"),
        None => format!("{provider}-{timestamp}"),
    }
}

pub fn build_unknown_step_error(provider: CloudProviderName, step_id: &str) -> Error {
    SystemError::UnknownJobStep {
        provider,
        step_id: step_id.to_string(),
    }
    .into()
}
