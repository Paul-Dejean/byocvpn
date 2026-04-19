use byocvpn_core::error::{Result, SystemError};
use log::*;

use crate::{routing::routes::remove_vpn_routes, tunnel_manager::TUNNEL_MANAGER};

pub async fn disconnect_vpn() -> Result<()> {
    info!("[VPN Disconnect] Disconnecting VPN tunnel...");

    let Some(mut handle) = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?
        .take()
    else {
        warn!("[VPN Disconnect] Disconnect requested but VPN is not connected, returning early.");
        return Ok(());
    };

    match handle.route_monitor_shutdown.send(()) {
        Ok(_) => debug!("[VPN Disconnect] Shutdown signal sent to route monitor."),
        Err(_) => warn!(
            "[VPN Disconnect] Failed to send shutdown signal (route monitor likely already stopped)."
        ),
    }
    match handle.route_monitor_task.await {
        Ok(_) => info!("[VPN Disconnect] Route monitor stopped."),
        Err(error) => warn!(
            "[VPN Disconnect] Route monitor task panicked while shutting down: {:?}",
            error
        ),
    }

    remove_vpn_routes(handle.interface_index, &handle.server_ip).await;
    info!("[VPN Disconnect] Removed VPN routes.");

    if let Some(mut dns_override_guard) =
        handle.dns_override_guard.take()
    {
        match dns_override_guard.restore_previous_dns_configuration() {
            Ok(_) => info!("[VPN Disconnect] Restored original DNS."),
            Err(error) => warn!("[VPN Disconnect] Failed to restore DNS: {error}"),
        }
    }

    match handle.shutdown.send(()) {
        Ok(_) => debug!("[VPN Disconnect] Shutdown signal sent to tunnel task."),
        Err(_) => warn!(
            "[VPN Disconnect] Failed to send shutdown signal (tunnel task likely already stopped)."
        ),
    }

    match handle.metrics_shutdown.send(()) {
        Ok(_) => debug!("[VPN Disconnect] Shutdown signal sent to metrics broadcaster."),
        Err(_) => warn!(
            "[VPN Disconnect] Failed to send shutdown signal (metrics broadcaster likely already stopped)."
        ),
    }

    match handle.task.await {
        Ok(_) => debug!("[VPN Disconnect] Tunnel task stopped."),
        Err(error) => error!(
            "[VPN Disconnect] Tunnel task panicked while shutting down: {:?}",
            error
        ),
    }

    match handle.metrics_task.await {
        Ok(_) => debug!("[VPN Disconnect] Metrics task stopped."),
        Err(error) => warn!(
            "[VPN Disconnect] Metrics task panicked while shutting down: {:?}",
            error
        ),
    }

    info!("[VPN Disconnect] VPN disconnected.");
    Ok(())
}
