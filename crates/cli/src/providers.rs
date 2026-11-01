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

    let mut configured_providers = context.providers.list_configured().await?;

    match configured_providers.len() {
        0 => Err(CliError::NoProviderConfigured),
        1 => configured_providers
            .pop()
            .ok_or(CliError::NoProviderConfigured),
        _ => {
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
                .ok_or(CliError::NoProviderConfigured)
        }
    }
}

pub async fn list_instances_across_providers(
    context: &AppContext,
    requested_provider: Option<CloudProviderName>,
    region: Option<&str>,
) -> CliResult<Vec<InstanceInfo>> {
    if requested_provider.is_none() && context.providers.list_configured().await?.is_empty() {
        return Err(CliError::NoProviderConfigured);
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
    with_spinner(
        "Fetching servers",
        instances::find_instance(context, instance_id),
    )
    .await?
    .ok_or_else(|| CliError::InstanceNotFound {
        instance_id: instance_id.to_string(),
    })
}

pub fn describe_instance(instance: &InstanceInfo) -> String {
    format!(
        "{}  {}  {}  {}",
        instance.provider, instance.id, instance.region, instance.public_ip_v4
    )
}
