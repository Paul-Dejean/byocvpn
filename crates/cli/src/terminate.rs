use byocvpn_core::{cloud_provider::InstanceInfo, commands};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    connect::run_disconnect,
    error::{CliError, CliResult},
    providers::{
        describe_instance, find_instance, list_instances_across_providers, terminate_server,
    },
};

pub async fn run_terminate(
    daemon_client: &UnixDaemonClient,
    instance_id: Option<String>,
) -> CliResult<()> {
    let instance = match instance_id {
        Some(instance_id) => find_instance(&instance_id).await?,
        None => choose_instance_to_terminate().await?,
    };

    disconnect_if_connected_to(daemon_client, &instance).await?;

    terminate_server(instance.provider, &instance.region, &instance.id).await
}

async fn choose_instance_to_terminate() -> CliResult<InstanceInfo> {
    let instances = list_instances_across_providers(None, None).await?;

    if instances.is_empty() {
        return Err(CliError::NoInstancesFound);
    }

    let labels: Vec<String> = instances.iter().map(describe_instance).collect();

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a server to terminate")
        .items(&labels)
        .default(0)
        .interact()?;

    instances
        .into_iter()
        .nth(selected_index)
        .ok_or(CliError::NoInstancesFound)
}

async fn disconnect_if_connected_to(
    daemon_client: &UnixDaemonClient,
    instance: &InstanceInfo,
) -> CliResult<()> {
    let status = commands::status::fetch_vpn_status(daemon_client).await?;

    let is_connected_to_instance = status.connected
        && status
            .instance
            .is_some_and(|connected| connected.instance_id == instance.id);

    if is_connected_to_instance {
        run_disconnect(daemon_client, false).await?;
    }
    Ok(())
}
