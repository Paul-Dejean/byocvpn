use async_trait::async_trait;
use log::{info, warn};

use crate::{
    app_context::AppContext,
    cloud_provider::{CloudProvider, JobStep},
    error::{Error, Result},
    jobs::{JobObserver, JobSteps, build_job_id, run_job_steps},
};

pub trait EnableRegionObserver: JobObserver {
    fn on_job_started(&self, job_id: &str, steps: &[JobStep]);
    fn on_job_completed(&self, region: &str);
    fn on_job_failed(&self, error: &Error);
}

pub async fn run_enable_region_job(
    provider: &dyn CloudProvider,
    context: &AppContext,
    region: &str,
    observer: &dyn EnableRegionObserver,
) -> Result<()> {
    let provider_name = provider.get_provider_name();
    let job_id = build_job_id(&provider_name, Some(region));
    let steps = provider.get_enable_region_steps(region);
    info!(
        "Enable region job {job_id} started with {} steps",
        steps.len()
    );
    observer.on_job_started(&job_id, &steps);

    let mut enable_region_steps = EnableRegionSteps { provider, region };
    match run_job_steps(&steps, &mut enable_region_steps, observer).await {
        Ok(()) => {
            context
                .provider_flags
                .mark_region_enabled(&provider_name, region);
            info!("Enable region job {job_id} completed");
            observer.on_job_completed(region);
            Ok(())
        }
        Err(error) => {
            warn!("Enable region job {job_id} failed: {error}");
            observer.on_job_failed(&error);
            Err(error)
        }
    }
}

struct EnableRegionSteps<'a> {
    provider: &'a dyn CloudProvider,
    region: &'a str,
}

#[async_trait]
impl JobSteps for EnableRegionSteps<'_> {
    async fn run_step(&mut self, step: &JobStep) -> Result<()> {
        self.provider
            .run_enable_region_step(&step.id, self.region)
            .await
    }
}
