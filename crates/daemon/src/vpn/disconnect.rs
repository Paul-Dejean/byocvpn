use std::net::SocketAddr;

use byocvpn_core::error::{Result, SystemError};
use ini::Ini;

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

    if let Ok(config) = Ini::load_from_file("wg0.conf") {
        if let Some(peer) = config.section(Some("Peer")) {
            if let Some(endpoint_str) = peer.get("Endpoint") {
                if let Ok(endpoint) = endpoint_str.parse::<SocketAddr>() {
                    remove_vpn_routes(tun_interface_name, &endpoint.ip().to_string()).await;
                    info!("[VPN Disconnect] Removed VPN routes.");
                }
            }
        }
    }

    let maybe_handle = {
        let mut manager_guard = TUNNEL_MANAGER
            .lock()
            .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;
        manager_guard.take()
    };

    if let Some(mut handle) = maybe_handle {
        info!("[VPN Disconnect] Stopping tunnel task...");

        #[cfg(any(target_os = "macos", target_os = "linux", windows))]
        if let Some(mut domain_name_system_override_guard) =
            handle.domain_name_system_override_guard.take()
        {
            if let Err(error) = domain_name_system_override_guard.restore_now() {
                error!("[VPN Disconnect] Warning: Failed to restore DNS: {error}");
            } else {
                info!("[VPN Disconnect] Restored original DNS.");
            }
        }

        if handle.shutdown.send(()).is_ok() {
            info!("[VPN Disconnect] Shutdown signal sent to tunnel task.");
        } else {
            error!(
                "[VPN Disconnect] Warning: Failed to send shutdown signal (tunnel task likely already stopped)."
            );
        }

        let _ = handle.metrics_shutdown.send(());
        info!("[VPN Disconnect] Metrics broadcaster stopped.");

        let _ = handle.route_monitor_shutdown.send(());
        info!("[VPN Disconnect] Route monitor stopped.");

        info!("[VPN Disconnect] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => info!("[VPN Disconnect] Tunnel task completed successfully."),
            Err(e) => error!("[VPN Disconnect] Error: Tunnel task failed: {:?}", e),
        }

        let _ = handle.metrics_task.await;
        let _ = handle.route_monitor_task.await;
    } else {
        info!("[VPN Disconnect] No active tunnel found.");
    }

    info!("[VPN Disconnect] VPN disconnected. Daemon continues running.");
    Ok(())
}
