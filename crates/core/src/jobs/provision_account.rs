use async_trait::async_trait;
use tracing::{Instrument, info, info_span, warn};

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
    let span = info_span!("provision_account_job", job_id = %job_id, provider = %provider_name);
    async move {
        info!(steps = steps.len(), "provision account job started");
        observer.on_job_started(&job_id, &steps);

        let mut provision_steps = ProvisionAccountSteps { provider };
        match run_job_steps(&steps, &mut provision_steps, observer).await {
            Ok(()) => {
                context.provider_flags.mark_provisioned(&provider_name);
                info!("provision account job completed");
                observer.on_job_completed(&provider_name);
                Ok(())
            }
            Err(error) => {
                warn!(error = %error.describe(), "provision account job failed");
                observer.on_job_failed(&error);
                Err(error)
            }
        }
    }
    .instrument(span)
    .await
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
