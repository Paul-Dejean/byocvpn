use std::io::IsTerminal;

use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::{InstanceInfo, InstanceState},
    daemon_client::fetch_vpn_status,
    session::{self, ConnectTarget},
    tunnel::ConnectedInstance,
};
use dialoguer::{Select, theme::ColorfulTheme};
use log::info;

use crate::{
    error::CliResult,
    progress::with_step,
    providers::{
        describe_instance, find_instance, list_instances_across_providers, terminate_server,
    },
    spawn::spawn_new_server,
};

pub async fn run_connect(context: &AppContext, instance_id: Option<String>) -> CliResult<()> {
    let instance = match instance_id {
        Some(instance_id) => find_instance(context, &instance_id).await?,
        None => choose_instance_or_spawn(context).await?,
    };

    with_step(
        &format!("Connecting to {} in {}", instance.id, instance.region),
        session::connect(
            context,
            context.daemon_client.as_ref(),
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

pub async fn run_disconnect(context: &AppContext, offer_to_terminate: bool) -> CliResult<()> {
    let status = fetch_vpn_status(context.daemon_client.as_ref()).await?;

    let Some(connected_instance) = status.instance.filter(|_| status.connected) else {
        info!("Not connected to any VPN");
        return Ok(());
    };

    with_step(
        &format!(
            "Disconnecting from {} in {}",
            connected_instance.instance_id, connected_instance.region
        ),
        session::disconnect(context.daemon_client.as_ref()),
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
