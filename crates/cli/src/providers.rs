use byocvpn_aws::{AwsCredentials, AwsProvider};
use byocvpn_azure::{AzureProvider, credentials::AzureCredentials};
use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName, InstanceInfo},
    commands,
    credentials::{CredentialStore, StoredCredentials},
};
use byocvpn_gcp::{GcpProvider, credentials::GcpCredentials};
use byocvpn_oracle::{OracleProvider, credentials::OracleCredentials};
use dialoguer::{Select, theme::ColorfulTheme};
use log::warn;

use crate::{
    error::{CliError, CliResult},
    progress::{with_spinner, with_step},
};

pub async fn create_cloud_provider(
    provider_name: CloudProviderName,
) -> CliResult<Box<dyn CloudProvider>> {
    let store = CredentialStore::load().await?;
    let provider: Box<dyn CloudProvider> = match provider_name {
        CloudProviderName::Aws => {
            Box::new(AwsProvider::new(AwsCredentials::from_store(&store)?.into()))
        }
        CloudProviderName::Azure => Box::new(AzureProvider::new(
            AzureCredentials::from_store(&store)?.into(),
        )?),
        CloudProviderName::Gcp => Box::new(GcpProvider::new(
            GcpCredentials::from_store(&store)?.into(),
        )?),
        CloudProviderName::Oracle => Box::new(OracleProvider::new(
            OracleCredentials::from_store(&store)?.into(),
        )),
    };
    Ok(provider)
}

pub async fn terminate_server(
    provider_name: CloudProviderName,
    region: &str,
    instance_id: &str,
) -> CliResult<()> {
    let provider = create_cloud_provider(provider_name).await?;
    with_step(
        &format!("Terminating {instance_id} in {region}"),
        commands::terminate::terminate_instance(&*provider, region, instance_id),
    )
    .await?;
    Ok(())
}

pub async fn resolve_provider_name(
    requested_provider: Option<CloudProviderName>,
) -> CliResult<CloudProviderName> {
    if let Some(provider_name) = requested_provider {
        return Ok(provider_name);
    }

    let mut configured_providers = list_configured_provider_names().await?;

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
    requested_provider: Option<CloudProviderName>,
    region: Option<&str>,
) -> CliResult<Vec<InstanceInfo>> {
    let (instances, failure_messages) = with_spinner(
        "Fetching servers",
        collect_instances(requested_provider, region),
    )
    .await?;

    for failure_message in failure_messages {
        warn!("{failure_message}");
    }
    Ok(instances)
}

async fn collect_instances(
    requested_provider: Option<CloudProviderName>,
    region: Option<&str>,
) -> CliResult<(Vec<InstanceInfo>, Vec<String>)> {
    let provider_names = match requested_provider {
        Some(provider_name) => vec![provider_name],
        None => list_configured_provider_names().await?,
    };

    if provider_names.is_empty() {
        return Err(CliError::NoProviderConfigured);
    }

    let mut all_instances = Vec::new();
    let mut failure_messages = Vec::new();
    for provider_name in provider_names {
        let provider = create_cloud_provider(provider_name.clone()).await?;
        match commands::list::list_instances(&*provider, region).await {
            Ok(instances) => all_instances.extend(instances),
            Err(error) => {
                failure_messages.push(format!("Could not list {provider_name} servers: {error}"));
            }
        }
    }
    Ok((all_instances, failure_messages))
}

async fn list_configured_provider_names() -> CliResult<Vec<CloudProviderName>> {
    let store = CredentialStore::load().await?;

    let credential_checks = [
        (
            CloudProviderName::Aws,
            AwsCredentials::from_store(&store).is_ok(),
        ),
        (
            CloudProviderName::Azure,
            AzureCredentials::from_store(&store).is_ok(),
        ),
        (
            CloudProviderName::Gcp,
            GcpCredentials::from_store(&store).is_ok(),
        ),
        (
            CloudProviderName::Oracle,
            OracleCredentials::from_store(&store).is_ok(),
        ),
    ];

    Ok(credential_checks
        .into_iter()
        .filter(|(_, has_credentials)| *has_credentials)
        .map(|(provider_name, _)| provider_name)
        .collect())
}

pub async fn find_instance(instance_id: &str) -> CliResult<InstanceInfo> {
    list_instances_across_providers(None, None)
        .await?
        .into_iter()
        .find(|instance| instance.id == instance_id)
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
