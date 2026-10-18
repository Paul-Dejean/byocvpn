use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName, InstanceInfo},
    commands,
    connectivity::wait_until_ready,
    crypto::generate_keypair,
    error::Result,
};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    error::{CliError, CliResult},
    progress::with_step,
    providers::{create_cloud_provider, resolve_provider_name},
};

pub async fn spawn_new_server(
    requested_provider: Option<CloudProviderName>,
    requested_region: Option<String>,
) -> CliResult<InstanceInfo> {
    let provider_name = resolve_provider_name(requested_provider).await?;
    let provider = create_cloud_provider(provider_name).await?;
    let region = match requested_region {
        Some(region) => region,
        None => choose_region(&*provider).await?,
    };
    spawn_server(&*provider, &region).await
}

async fn spawn_server(provider: &dyn CloudProvider, region: &str) -> CliResult<InstanceInfo> {
    with_step(
        &format!("Preparing {}", region),
        prepare_region(provider, region),
    )
    .await?;

    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let instance = with_step(
        &format!("Launching server in {}", region),
        commands::spawn::launch_instance(
            provider,
            region,
            "",
            &server_private_key,
            &client_public_key,
        ),
    )
    .await?;

    with_step(
        "Waiting for the server to accept connections",
        wait_until_ready(&instance.public_ip_v4),
    )
    .await?;

    commands::spawn::write_wireguard_config(
        &provider.get_provider_name(),
        region,
        &instance,
        &client_private_key,
        &server_public_key,
    )
    .await?;

    Ok(instance)
}

async fn prepare_region(provider: &dyn CloudProvider, region: &str) -> Result<()> {
    commands::setup::setup(provider).await?;
    commands::setup::enable_region(provider, region).await
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
