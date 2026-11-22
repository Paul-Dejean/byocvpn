use byocvpn_core::error::Result;
use log::{debug, error, info, warn};
use tokio::{sync::watch::Sender, task::JoinHandle};

use crate::{
    firewall,
    routing::{dns::DnsOverrideGuard, routes::remove_vpn_routes},
    tunnel_manager::{TunnelHandle, lock_tunnel_manager},
    vpn::session,
};

pub async fn disconnect_vpn() -> Result<()> {
    info!("[VPN Disconnect] Disconnecting VPN tunnel...");

    let Some(handle) = lock_tunnel_manager()?.take() else {
        warn!(
            "[VPN Disconnect] No active tunnel in memory (daemon restarted?), clearing firewall rules."
        );
        clear_system_state();
        return Ok(());
    };
    let TunnelHandle {
        shutdown,
        task,
        metrics_task,
        metrics_shutdown,
        route_monitor_task,
        route_monitor_shutdown,
        dns_override_guard,
        server_ip,
        interface_index,
        ..
    } = handle;

    stop_route_monitor(route_monitor_shutdown, route_monitor_task).await;
    remove_vpn_routes(interface_index, &server_ip).await;
    info!("[VPN Disconnect] Removed VPN routes.");
    restore_dns(dns_override_guard);
    stop_tunnel_tasks(shutdown, task, metrics_shutdown, metrics_task).await;
    clear_system_state();

    info!("[VPN Disconnect] VPN disconnected.");
    Ok(())
}

async fn stop_route_monitor(shutdown: Sender<()>, task: JoinHandle<()>) {
    send_shutdown_signal(&shutdown, "route monitor");
    match task.await {
        Ok(()) => info!("[VPN Disconnect] Route monitor stopped."),
        Err(error) => {
            warn!("[VPN Disconnect] Route monitor task panicked while shutting down: {error:?}");
        }
    }
}

fn restore_dns(dns_override_guard: Option<DnsOverrideGuard>) {
    let Some(mut guard) = dns_override_guard else {
        return;
    };
    match guard.restore_previous_dns_configuration() {
        Ok(()) => info!("[VPN Disconnect] Restored original DNS."),
        Err(error) => warn!("[VPN Disconnect] Failed to restore DNS: {error}"),
    }
}

async fn stop_tunnel_tasks(
    shutdown: Sender<()>,
    task: JoinHandle<()>,
    metrics_shutdown: Sender<()>,
    metrics_task: JoinHandle<()>,
) {
    send_shutdown_signal(&shutdown, "tunnel task");
    send_shutdown_signal(&metrics_shutdown, "metrics broadcaster");

    match task.await {
        Ok(()) => debug!("[VPN Disconnect] Tunnel task stopped."),
        Err(error) => {
            error!("[VPN Disconnect] Tunnel task panicked while shutting down: {error:?}");
        }
    }
    match metrics_task.await {
        Ok(()) => debug!("[VPN Disconnect] Metrics task stopped."),
        Err(error) => {
            warn!("[VPN Disconnect] Metrics task panicked while shutting down: {error:?}");
        }
    }
}

fn send_shutdown_signal(sender: &Sender<()>, target: &str) {
    match sender.send(()) {
        Ok(()) => debug!("[VPN Disconnect] Shutdown signal sent to {target}."),
        Err(_) => warn!(
            "[VPN Disconnect] Failed to send shutdown signal ({target} likely already stopped)."
        ),
    }
}

fn clear_system_state() {
    if let Err(error) = firewall::remove() {
        warn!("[VPN Disconnect] Kill switch removal failed: {error}");
    }
    if let Err(error) = session::clear_session() {
        warn!("[VPN Disconnect] Failed to clear session file: {error}");
    }
}
