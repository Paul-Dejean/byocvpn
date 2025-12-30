use std::net::SocketAddr;

use byocvpn_core::error::{Result, SystemError};
use ini::Ini;

use crate::{routing::routes::remove_vpn_routes, tunnel_manager::TUNNEL_MANAGER};

pub async fn disconnect_vpn() -> Result<()> {
    println!("[VPN Disconnect] Disconnecting VPN tunnel...");

    // Check if config file exists to get endpoint info for route cleanup
    if let Ok(config) = Ini::load_from_file("wg0.conf") {
        if let Some(peer) = config.section(Some("Peer")) {
            if let Some(endpoint_str) = peer.get("Endpoint") {
                if let Ok(endpoint) = endpoint_str.parse::<SocketAddr>() {
                    remove_vpn_routes("utun4", &endpoint.ip().to_string()).await;
                    println!("[VPN Disconnect] Removed VPN routes.");
                }
            }
        }
    }

    // Shut down the tunnel task
    let maybe_handle = {
        let mut manager_guard = TUNNEL_MANAGER
            .lock()
            .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;
        manager_guard.take()
    };

    if let Some(mut handle) = maybe_handle {
        println!("[VPN Disconnect] Stopping tunnel task...");

        #[cfg(target_os = "macos")]
        if let Some(mut domain_name_system_override_guard) =
            handle.domain_name_system_override_guard.take()
        {
            if let Err(error) = domain_name_system_override_guard.restore_now() {
                eprintln!("[VPN Disconnect] Warning: Failed to restore DNS: {error}");
            } else {
                println!("[VPN Disconnect] Restored original DNS.");
            }
        }

        // Send the shutdown signal via the watch channel
        if handle.shutdown.send(()).is_ok() {
            println!("[VPN Disconnect] Shutdown signal sent to tunnel task.");
        } else {
            eprintln!(
                "[VPN Disconnect] Warning: Failed to send shutdown signal (tunnel task likely already stopped)."
            );
        }

        // Stop metrics broadcaster
        let _ = handle.metrics_shutdown.send(());
        println!("[VPN Disconnect] Metrics broadcaster stopped.");

        // Wait for the tunnel task to complete
        println!("[VPN Disconnect] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => println!("[VPN Disconnect] Tunnel task completed successfully."),
            Err(e) => eprintln!("[VPN Disconnect] Error: Tunnel task failed: {:?}", e),
        }

        // Wait for metrics task
        let _ = handle.metrics_task.await;
    } else {
        println!("[VPN Disconnect] No active tunnel found.");
    }

    println!("[VPN Disconnect] VPN disconnected. Daemon continues running.");
    Ok(())
}
