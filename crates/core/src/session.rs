use log::info;

use crate::{
    app_context::AppContext,
    cloud_provider::CloudProviderName,
    config::get_wireguard_config_file_path,
    daemon_client::{DaemonClient, DaemonCommand, VpnConnectParams, fetch_vpn_status},
    error::{DaemonError, Result},
    wireguard_config::parse_wireguard_config,
};

pub struct ConnectTarget {
    pub provider: CloudProviderName,
    pub region: String,
    pub instance_id: String,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
}

pub async fn connect(
    context: &AppContext,
    daemon_client: &dyn DaemonClient,
    target: ConnectTarget,
) -> Result<()> {
    reject_if_already_connected(daemon_client).await?;

    let kill_switch_enabled = context.settings.load_vpn_settings().session_killswitch;
    let wireguard_file_path =
        get_wireguard_config_file_path(&target.provider, &target.region, &target.instance_id)
            .await?;
    let wireguard_config = parse_wireguard_config(&wireguard_file_path.to_string_lossy()).await?;

    info!("Sending connect command to daemon...");
    let response = daemon_client
        .send_command(DaemonCommand::Connect(Box::new(VpnConnectParams {
            instance_id: target.instance_id,
            private_key: wireguard_config.private_key,
            public_key: wireguard_config.public_key,
            server_endpoint: wireguard_config.server_endpoint,
            private_ipv4: wireguard_config.private_ipv4,
            private_ipv6: wireguard_config.private_ipv6,
            dns_servers: wireguard_config.dns_servers,
            region: target.region,
            provider: target.provider,
            public_ip_v4: target.public_ip_v4,
            public_ip_v6: target.public_ip_v6,
            kill_switch_enabled,
        })))
        .await?;
    info!("Daemon response: {response}");
    Ok(())
}

pub async fn disconnect(daemon_client: &dyn DaemonClient) -> Result<()> {
    if !daemon_client.is_daemon_running().await {
        return Err(DaemonError::NotRunning.into());
    }
    daemon_client
        .send_command(DaemonCommand::Disconnect)
        .await?;
    Ok(())
}

async fn reject_if_already_connected(daemon_client: &dyn DaemonClient) -> Result<()> {
    let status = fetch_vpn_status(daemon_client).await?;
    match status.instance.filter(|_| status.connected) {
        Some(connected_instance) => Err(DaemonError::AlreadyConnected {
            instance_id: connected_instance.instance_id,
        }
        .into()),
        None => Ok(()),
    }
}
