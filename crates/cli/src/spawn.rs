use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::{CloudProvider, CloudProviderName, InstanceInfo},
    commands,
    jobs::{region_preparation::ensure_region_ready, spawn::run_spawn_job},
};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    error::{CliError, CliResult},
    progress::TerminalJobObserver,
    providers::resolve_provider_name,
};

pub async fn spawn_new_server(
    context: &AppContext,
    requested_provider: Option<CloudProviderName>,
    requested_region: Option<String>,
) -> CliResult<InstanceInfo> {
    let provider_name = resolve_provider_name(context, requested_provider).await?;
    let provider = context.providers.create(provider_name).await?;
    let region = match requested_region {
        Some(region) => region,
        None => choose_region(&*provider).await?,
    };

    let observer = TerminalJobObserver::default();
    ensure_region_ready(&*provider, context, &region, &observer).await?;
    let instance = run_spawn_job(&*provider, context, &region, &observer).await?;
    Ok(instance)
}

async fn choose_region(provider: &dyn CloudProvider) -> CliResult<String> {
    let regions = commands::setup::get_regions(provider).await?;
    let labels: Vec<String> = regions
        .iter()
        .map(|region| format!("{} ({})", region.name, region.country))
        .collect();

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a region for the new server")
        .items(&labels)
        .default(0)
        .interact()?;

    regions
        .into_iter()
        .nth(selected_index)
        .map(|region| region.name)
        .ok_or(CliError::RegionRequired)
}
