use std::io::IsTerminal;

use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::{InstanceInfo, InstanceState},
    commands,
    session::{self, ConnectTarget},
    tunnel::ConnectedInstance,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use dialoguer::{Select, theme::ColorfulTheme};
use log::info;

use crate::{
    error::{CliError, CliResult},
    progress::with_step,
    providers::{
        describe_instance, find_instance, list_instances_across_providers, terminate_server,
    },
    spawn::spawn_new_server,
};

pub async fn run_connect(
    context: &AppContext,
    daemon_client: &UnixDaemonClient,
    instance_id: Option<String>,
) -> CliResult<()> {
    reject_if_already_connected(daemon_client).await?;

    let instance = match instance_id {
        Some(instance_id) => find_instance(context, &instance_id).await?,
        None => choose_instance_or_spawn(context).await?,
    };

    with_step(
        &format!("Connecting to {} in {}", instance.id, instance.region),
        session::connect(
            context,
            daemon_client,
            ConnectTarget {
                provider: instance.provider.clone(),
                region: instance.region.clone(),
                instance_id: instance.id.clone(),
                public_ip_v4: Some(instance.public_ip_v4.clone()),
                public_ip_v6: Some(instance.public_ip_v6.clone()),
            },
        ),
    )
    .await?;

    Ok(())
}

pub async fn run_disconnect(
    context: &AppContext,
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
        session::disconnect(daemon_client),
    )
    .await?;

    if offer_to_terminate && confirm_termination(&connected_instance)? {
        terminate_server(
            context,
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

async fn choose_instance_or_spawn(context: &AppContext) -> CliResult<InstanceInfo> {
    let running_instances: Vec<InstanceInfo> = list_instances_across_providers(context, None, None)
        .await?
        .into_iter()
        .filter(|instance| instance.state == InstanceState::Running)
        .collect();

    if running_instances.is_empty() {
        info!("No running servers found, spawning a new one");
        return spawn_new_server(context, None, None).await;
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
        None => spawn_new_server(context, None, None).await,
    }
}
