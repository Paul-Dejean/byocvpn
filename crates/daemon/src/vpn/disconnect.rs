use byocvpn_core::error::{Result, SystemError};

use crate::{routing::routes::remove_vpn_routes, tunnel_manager::TUNNEL_MANAGER};
use log::*;

pub async fn disconnect_vpn() -> Result<()> {
    info!("[VPN Disconnect] Disconnecting VPN tunnel...");

    #[cfg(target_os = "macos")]
    let tun_interface_name = "utun4";
    #[cfg(windows)]
    let tun_interface_name = "byocvpn";
    #[cfg(not(any(target_os = "macos", windows)))]
    let tun_interface_name = "tun0";

    let maybe_handle = {
        let mut manager_guard = TUNNEL_MANAGER
            .lock()
            .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;
        manager_guard.take()
    };

    if let Some(mut handle) = maybe_handle {
        // Stop the route monitor before deleting routes so it doesn't re-add them.
        let _ = handle.route_monitor_shutdown.send(());
        let _ = handle.route_monitor_task.await;
        debug!("[VPN Disconnect] Route monitor stopped.");

        remove_vpn_routes(tun_interface_name, &handle.server_ip).await;
        info!("[VPN Disconnect] Removed VPN routes.");

        if let Some(mut domain_name_system_override_guard) =
            handle.domain_name_system_override_guard.take()
        {
            if let Err(error) = domain_name_system_override_guard.restore_now() {
                warn!("[VPN Disconnect] Failed to restore DNS: {error}");
            } else {
                info!("[VPN Disconnect] Restored original DNS.");
            }
        }

        if handle.shutdown.send(()).is_ok() {
            debug!("[VPN Disconnect] Shutdown signal sent to tunnel task.");
        } else {
            warn!("[VPN Disconnect] Failed to send shutdown signal (tunnel task likely already stopped).");
        }

        let _ = handle.metrics_shutdown.send(());
        debug!("[VPN Disconnect] Metrics broadcaster stopped.");

        debug!("[VPN Disconnect] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => debug!("[VPN Disconnect] Tunnel task completed successfully."),
            Err(error) => error!("[VPN Disconnect] Tunnel task failed: {:?}", error),
        }

        if let Err(error) = handle.metrics_task.await {
            warn!("[VPN Disconnect] Metrics task panicked: {:?}", error);
        }
    } else {
        info!("[VPN Disconnect] No active tunnel found.");
    }

    info!("[VPN Disconnect] VPN disconnected. Daemon continues running.");
    Ok(())
}
