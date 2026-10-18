use byocvpn_core::tunnel::TunnelMetrics;

use crate::tunnel_manager::TUNNEL_MANAGER;

pub async fn get_current_metrics() -> Option<TunnelMetrics> {
    let shared_metrics = {
        let manager = TUNNEL_MANAGER.lock().ok()?;
        manager.as_ref()?.metrics.clone()
    };

    let current_metrics = shared_metrics.read().await.clone();
    Some(current_metrics)
}
