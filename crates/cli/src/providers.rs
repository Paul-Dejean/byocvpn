use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::{CloudProviderName, InstanceInfo},
    instances,
};
use dialoguer::{Select, theme::ColorfulTheme};
use log::warn;

use crate::{
    error::{CliError, CliResult},
    progress::{with_spinner, with_step},
};

pub async fn terminate_server(
    context: &AppContext,
    provider_name: CloudProviderName,
    region: &str,
    instance_id: &str,
) -> CliResult<()> {
    with_step(
        &format!("Terminating {instance_id} in {region}"),
        instances::terminate_instance(context, provider_name, region, instance_id),
    )
    .await?;
    Ok(())
}

pub async fn resolve_provider_name(
    context: &AppContext,
    requested_provider: Option<CloudProviderName>,
) -> CliResult<CloudProviderName> {
    if let Some(provider_name) = requested_provider {
        return Ok(provider_name);
    }

    let mut configured_providers = context.providers.require_configured().await?;
    if configured_providers.len() == 1 {
        return configured_providers
            .pop()
            .ok_or(CliError::SelectionOutOfRange);
    }

    let labels: Vec<String> = configured_providers
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a cloud provider")
        .items(&labels)
        .default(0)
        .interact()?;

    configured_providers
        .into_iter()
        .nth(selected_index)
        .ok_or(CliError::SelectionOutOfRange)
}

pub async fn list_instances_across_providers(
    context: &AppContext,
    requested_provider: Option<CloudProviderName>,
    region: Option<&str>,
) -> CliResult<Vec<InstanceInfo>> {
    if requested_provider.is_none() {
        context.providers.require_configured().await?;
    }

    let listing = with_spinner(
        "Fetching servers",
        instances::list_instances(context, requested_provider, region),
    )
    .await?;

    for failure in &listing.failures {
        warn!(
            "Could not list {} servers: {}",
            failure.provider, failure.error
        );
    }
    Ok(listing.instances)
}

pub async fn find_instance(context: &AppContext, instance_id: &str) -> CliResult<InstanceInfo> {
    Ok(with_spinner(
        "Fetching servers",
        instances::require_instance(context, instance_id),
    )
    .await?)
}

pub fn describe_instance(instance: &InstanceInfo) -> String {
    format!(
        "{}  {}  {}  {}",
        instance.provider, instance.id, instance.region, instance.public_ip_v4
    )
}
