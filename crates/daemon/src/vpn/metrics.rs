use std::sync::Arc;

use byocvpn_core::tunnel::TunnelMetrics;

use crate::tunnel_manager::TUNNEL_MANAGER;

pub async fn get_current_metrics() -> Option<TunnelMetrics> {
    let shared_metrics = {
        let manager = TUNNEL_MANAGER.lock().ok()?;
        Arc::clone(&manager.as_ref()?.metrics)
    };

    let current_metrics = *shared_metrics.read().await;
    Some(current_metrics)
}
