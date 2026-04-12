use serde::{Deserialize, Serialize};

use crate::{
    cloud_provider::{CloudProvider, SpawnStep, SpawnStepStatus},
    error::Result,
};

pub async fn setup(provider: &dyn CloudProvider) -> Result<()> {
    provider.setup().await?;
    Ok(())
}

pub async fn enable_region(provider: &dyn CloudProvider, region: &str) -> Result<()> {
    provider.enable_region(region).await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub name: String,
    pub country: String,
}

pub async fn get_regions(provider: &dyn CloudProvider) -> Result<Vec<Region>> {
    provider.get_regions().await
}

pub async fn run_provision_account_steps<F>(
    provider: &dyn CloudProvider,
    steps: &[SpawnStep],
    on_step_progress: F,
) -> Result<()>
where
    F: Fn(&str, SpawnStepStatus, Option<String>),
{
    for step in steps {
        on_step_progress(&step.id, SpawnStepStatus::Running, None);
        if let Err(error) = provider.run_provision_account_step(&step.id).await {
            on_step_progress(&step.id, SpawnStepStatus::Failed, Some(error.to_string()));
            return Err(error);
        }
        on_step_progress(&step.id, SpawnStepStatus::Completed, None);
    }
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
    for step in steps {
        on_step_progress(&step.id, SpawnStepStatus::Running, None);
        if let Err(error) = provider.run_enable_region_step(&step.id, region).await {
            on_step_progress(&step.id, SpawnStepStatus::Failed, Some(error.to_string()));
            return Err(error);
        }
        on_step_progress(&step.id, SpawnStepStatus::Completed, None);
    }
    Ok(())
}
