use byocvpn_core::error::Result;
use tokio::{sync::watch::Sender, task::JoinHandle};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    firewall,
    routing::{dns::DnsOverrideGuard, routes::remove_vpn_routes},
    tunnel_manager::{TunnelHandle, lock_tunnel_manager},
    vpn::session,
};

#[instrument(name = "disconnect", skip_all)]
pub async fn disconnect_vpn() -> Result<()> {
    info!("disconnecting");

    let Some(handle) = lock_tunnel_manager()?.take() else {
        warn!("no active tunnel in memory, clearing firewall rules");
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
    debug!("removed VPN routes");
    restore_dns(dns_override_guard);
    stop_tunnel_tasks(shutdown, task, metrics_shutdown, metrics_task).await;
    clear_system_state();

    info!("disconnected");
    Ok(())
}

async fn stop_route_monitor(shutdown: Sender<()>, task: JoinHandle<()>) {
    send_shutdown_signal(&shutdown, "route monitor");
    match task.await {
        Ok(()) => debug!("route monitor stopped"),
        Err(error) => {
            warn!(?error, "route monitor task panicked while shutting down");
        }
    }
}

fn restore_dns(dns_override_guard: Option<DnsOverrideGuard>) {
    let Some(mut guard) = dns_override_guard else {
        return;
    };
    match guard.restore_previous_dns_configuration() {
        Ok(()) => info!("dns restored"),
        Err(error) => warn!(%error, "failed to restore DNS"),
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
        Ok(()) => debug!("tunnel task stopped"),
        Err(error) => {
            error!(?error, "tunnel task panicked while shutting down");
        }
    }
    match metrics_task.await {
        Ok(()) => debug!("metrics task stopped"),
        Err(error) => {
            warn!(?error, "metrics task panicked while shutting down");
        }
    }
}

fn send_shutdown_signal(sender: &Sender<()>, target: &str) {
    if sender.send(()).is_ok() {
        debug!("shutdown signal sent to {target}");
    } else {
        debug!("failed to send shutdown signal ({target} likely already stopped)");
    }
}

fn clear_system_state() {
    if let Err(error) = firewall::remove() {
        warn!(%error, "kill switch removal failed");
    }
    if let Err(error) = session::clear_session() {
        warn!(%error, "failed to clear session file");
    }
}
