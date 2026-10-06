use byocvpn_core::{
    commands,
    error::Result,
    metrics_stream,
    tunnel::{ConnectedInstance, VpnStatus},
};
use byocvpn_daemon::{constants::metrics_socket_path, daemon_client::UnixDaemonClient};
use tauri::AppHandle;

use crate::{ledger_store::LedgerStore, vpn_backend::publish_vpn_status};

const TUNNEL_DROPPED_MESSAGE: &str = "VPN tunnel dropped. Kill switch is blocking all traffic.";

pub fn create_daemon_client(_app_handle: &AppHandle) -> UnixDaemonClient {
    UnixDaemonClient
}

pub async fn start_status_stream(
    app_handle: &AppHandle,
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
) -> Result<()> {
    let status_handle = app_handle.clone();
    let last_connected = connected_instance.clone();
    metrics_stream::start(
        metrics_socket_path(),
        connected_instance,
        connected_at,
        move |vpn_status| {
            publish_vpn_status(
                &status_handle,
                &mark_dropped_tunnel(vpn_status, &last_connected),
            );
        },
    )
    .await
}

pub async fn start_status_subscription(
    app_handle: &AppHandle,
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
) -> Result<()> {
    let instance_id = connected_instance.instance_id.clone();
    let status_handle = app_handle.clone();
    let ledger_handle = app_handle.clone();
    let last_connected = connected_instance.clone();

    commands::subscribe::start_metrics_subscription(
        metrics_socket_path(),
        connected_instance,
        connected_at,
        move |vpn_status| {
            publish_vpn_status(
                &status_handle,
                &mark_dropped_tunnel(vpn_status, &last_connected),
            );
        },
        move |bytes_sent, bytes_received| {
            if let Some(ledger) = LedgerStore::open(&ledger_handle) {
                ledger.update_metrics(&instance_id, bytes_sent, bytes_received);
            }
        },
    )
    .await
}

pub async fn stop_status_stream() -> Result<()> {
    metrics_stream::stop().await
}

fn mark_dropped_tunnel(vpn_status: VpnStatus, last_connected: &ConnectedInstance) -> VpnStatus {
    if vpn_status.connected {
        return vpn_status;
    }
    VpnStatus {
        connected: true,
        instance: Some(last_connected.clone()),
        metrics: None,
        connected_at: None,
        connection_error: Some(TUNNEL_DROPPED_MESSAGE.to_string()),
    }
}
