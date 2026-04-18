use log::*;
use serde::{Deserialize, Serialize};

use crate::{
    cloud_provider::{CloudProvider, SpawnStep, SpawnStepStatus},
    error::Result,
};

pub async fn setup(provider: &dyn CloudProvider) -> Result<()> {
    info!("Running provider account setup");
    provider.setup().await?;
    info!("Provider account setup completed");
    Ok(())
}

pub async fn enable_region(provider: &dyn CloudProvider, region: &str) -> Result<()> {
    info!("Enabling region: {}", region);
    provider.enable_region(region).await?;
    info!("Region enabled: {}", region);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub name: String,
    pub country: String,
}

pub async fn get_regions(provider: &dyn CloudProvider) -> Result<Vec<Region>> {
    debug!("Fetching available regions from provider");
    let regions = provider.get_regions().await?;
    debug!("Fetched {} regions", regions.len());
    Ok(regions)
}

pub async fn run_provision_account_steps<F>(
    provider: &dyn CloudProvider,
    steps: &[SpawnStep],
    on_step_progress: F,
) -> Result<()>
where
    F: Fn(&str, SpawnStepStatus, Option<String>),
{
    info!("Running {} provision account steps", steps.len());
    for step in steps {
        debug!("Running provision account step: {}", step.id);
        on_step_progress(&step.id, SpawnStepStatus::Running, None);
        if let Err(error) = provider.run_provision_account_step(&step.id).await {
            error!(
                "Provision account step failed: step={} error={}",
                step.id, error
            );
            on_step_progress(&step.id, SpawnStepStatus::Failed, Some(error.to_string()));
            return Err(error);
        }
        debug!("Provision account step completed: {}", step.id);
        on_step_progress(&step.id, SpawnStepStatus::Completed, None);
    }
    info!("All provision account steps completed");
    Ok(())
}

pub async fn run_enable_region_steps<F>(
    provider: &dyn CloudProvider,
    steps: &[SpawnStep],
    region: &str,
    on_step_progress: F,
) -> Result<()>
where
    F: Fn(&str, SpawnStepStatus, Option<String>),
{
    info!(
        "Running {} enable-region steps for region: {}",
        steps.len(),
        region
    );
    for step in steps {
        debug!(
            "Running enable-region step: {} for region: {}",
            step.id, region
        );
        on_step_progress(&step.id, SpawnStepStatus::Running, None);
        if let Err(error) = provider.run_enable_region_step(&step.id, region).await {
            error!(
                "Enable-region step failed: step={} region={} error={}",
                step.id, region, error
            );
            on_step_progress(&step.id, SpawnStepStatus::Failed, Some(error.to_string()));
            return Err(error);
        }
        debug!(
            "Enable-region step completed: {} for region: {}",
            step.id, region
        );
        on_step_progress(&step.id, SpawnStepStatus::Completed, None);
    }
    info!("All enable-region steps completed for region: {}", region);
    Ok(())
}
