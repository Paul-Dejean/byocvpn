use async_trait::async_trait;
use log::{info, warn};

use crate::{
    app_context::AppContext,
    cloud_provider::{CloudProvider, CloudProviderName, JobStep},
    error::{Error, Result},
    jobs::{JobObserver, JobSteps, build_job_id, run_job_steps},
};

pub trait ProvisionAccountObserver: JobObserver {
    fn on_job_started(&self, job_id: &str, steps: &[JobStep]);
    fn on_job_completed(&self, provider: &CloudProviderName);
    fn on_job_failed(&self, error: &Error);
}

pub async fn run_provision_account_job(
    provider: &dyn CloudProvider,
    context: &AppContext,
    observer: &dyn ProvisionAccountObserver,
) -> Result<()> {
    let provider_name = provider.get_provider_name();
    let job_id = build_job_id(&provider_name, None);
    let steps = provider.get_provision_account_steps();
    info!(
        "Provision account job {job_id} started with {} steps",
        steps.len()
    );
    observer.on_job_started(&job_id, &steps);

    let mut provision_steps = ProvisionAccountSteps { provider };
    match run_job_steps(&steps, &mut provision_steps, observer).await {
        Ok(()) => {
            context.provider_flags.mark_provisioned(&provider_name);
            info!("Provision account job {job_id} completed");
            observer.on_job_completed(&provider_name);
            Ok(())
        }
        Err(error) => {
            warn!("Provision account job {job_id} failed: {error}");
            observer.on_job_failed(&error);
            Err(error)
        }
    }
}

struct ProvisionAccountSteps<'a> {
    provider: &'a dyn CloudProvider,
}

#[async_trait]
impl JobSteps for ProvisionAccountSteps<'_> {
    async fn run_step(&mut self, step: &JobStep) -> Result<()> {
        self.provider.run_provision_account_step(&step.id).await
    }
}
