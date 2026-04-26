use byocvpn_core::{
    error::{Result, SystemError},
    tunnel::VpnStatus,
};

use crate::tunnel_manager::TUNNEL_MANAGER;

pub async fn get_vpn_status() -> Result<VpnStatus> {
    let manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;

    if let Some(handle) = manager.as_ref() {
        let is_running = !handle.task.is_finished();

        let connected_at = handle
            .connected_at
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_secs());

        Ok(VpnStatus {
            connected: is_running,
            instance: handle.instance.clone(),
            metrics: Some(handle.metrics.read().await.clone()),
            connected_at,
        })
    } else {
        Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
            connected_at: None,
        })
    }
}
