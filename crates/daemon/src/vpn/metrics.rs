use byocvpn_core::tunnel::TunnelMetrics;

use crate::tunnel_manager::TUNNEL_MANAGER;

pub async fn get_current_metrics() -> Option<TunnelMetrics> {
    let manager = TUNNEL_MANAGER.lock().ok()?;
    if let Some(handle) = manager.as_ref() {
        let metrics = handle.metrics.read().await;
        Some(metrics.clone())
    } else {
        None
    }
}
