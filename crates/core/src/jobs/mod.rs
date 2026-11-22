pub mod enable_region;
pub mod provision_account;
pub mod region_preparation;
pub mod spawn;

use async_trait::async_trait;
use jiff::Timestamp;
use tracing::{Instrument, debug_span};

use crate::{
    cloud_provider::{CloudProviderName, JobStep, JobStepStatus},
    error::{ErrorPayload, Result},
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
        observer.on_step_changed(step, JobStepStatus::Running, None);

        let step_outcome = job_steps
            .run_step(step)
            .instrument(debug_span!("step", id = %step.id))
            .await;
        if let Err(step_error) = step_outcome {
            observer.on_step_changed(step, JobStepStatus::Failed, Some(step_error.to_payload()));
            return Err(step_error);
        }

        observer.on_step_changed(step, JobStepStatus::Completed, None);
        job_steps.on_step_completed(step);
    }
    Ok(())
}

pub fn build_job_id(provider: &CloudProviderName, region: Option<&str>) -> String {
    let timestamp = Timestamp::now().as_millisecond();
    region.map_or_else(
        || format!("{provider}-{timestamp}"),
        |region| format!("{provider}-{region}-{timestamp}"),
    )
}
