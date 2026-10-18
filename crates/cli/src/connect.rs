use std::io::IsTerminal;

use byocvpn_core::{
    cloud_provider::{InstanceInfo, InstanceState},
    commands,
    tunnel::ConnectedInstance,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use dialoguer::{Select, theme::ColorfulTheme};
use log::info;

use crate::{
    error::{CliError, CliResult},
    progress::with_step,
    providers::{
        create_cloud_provider, describe_instance, find_instance, list_instances_across_providers,
        terminate_server,
    },
    spawn::spawn_new_server,
};

pub async fn run_connect(
    daemon_client: &UnixDaemonClient,
    instance_id: Option<String>,
) -> CliResult<()> {
    reject_if_already_connected(daemon_client).await?;

    let instance = match instance_id {
        Some(instance_id) => find_instance(&instance_id).await?,
        None => choose_instance_or_spawn().await?,
    };

    let provider = create_cloud_provider(instance.provider.clone()).await?;

    with_step(
        &format!("Connecting to {} in {}", instance.id, instance.region),
        commands::connect::connect(
            &*provider,
            daemon_client,
            &instance.region,
            &instance.id,
            Some(instance.public_ip_v4.clone()),
            Some(instance.public_ip_v6.clone()),
            true,
        ),
    )
    .await?;

    Ok(())
}

pub async fn run_disconnect(
    daemon_client: &UnixDaemonClient,
    offer_to_terminate: bool,
) -> CliResult<()> {
    let status = commands::status::fetch_vpn_status(daemon_client).await?;

    let Some(connected_instance) = status.instance.filter(|_| status.connected) else {
        info!("Not connected to any VPN");
        return Ok(());
    };

    with_step(
        &format!(
            "Disconnecting from {} in {}",
            connected_instance.instance_id, connected_instance.region
        ),
        commands::disconnect::disconnect(daemon_client),
    )
    .await?;

    if offer_to_terminate && confirm_termination(&connected_instance)? {
        terminate_server(
            connected_instance.provider,
            &connected_instance.region,
            &connected_instance.instance_id,
        )
        .await?;
    }

    Ok(())
}

fn confirm_termination(connected_instance: &ConnectedInstance) -> CliResult<bool> {
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Terminate {} in {} as well?",
            connected_instance.instance_id, connected_instance.region
        ))
        .items(["No", "Yes"])
        .default(0)
        .interact()?;

    Ok(selected_index == 1)
}

async fn reject_if_already_connected(daemon_client: &UnixDaemonClient) -> CliResult<()> {
    let status = commands::status::fetch_vpn_status(daemon_client).await?;

    match status.instance.filter(|_| status.connected) {
        Some(connected_instance) => Err(CliError::AlreadyConnected {
            instance_id: connected_instance.instance_id,
        }),
        None => Ok(()),
    }
}

async fn choose_instance_or_spawn() -> CliResult<InstanceInfo> {
    let running_instances: Vec<InstanceInfo> = list_instances_across_providers(None, None)
        .await?
        .into_iter()
        .filter(|instance| instance.state == InstanceState::Running)
        .collect();

    if running_instances.is_empty() {
        info!("No running servers found, spawning a new one");
        return spawn_new_server(None, None).await;
    }

    let mut labels: Vec<String> = running_instances.iter().map(describe_instance).collect();
    labels.push("Spawn a new server".to_string());

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a server to connect to")
        .items(&labels)
        .default(0)
        .interact()?;

    match running_instances.into_iter().nth(selected_index) {
        Some(instance) => Ok(instance),
        None => spawn_new_server(None, None).await,
    }
}
