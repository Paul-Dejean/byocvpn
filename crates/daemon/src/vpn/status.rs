use std::{sync::Arc, time::SystemTime};

use byocvpn_core::{
    error::{Result, SystemError},
    tunnel::{ConnectedInstance, TunnelMetrics, VpnStatus},
};
use tokio::sync::RwLock;

use crate::tunnel_manager::TUNNEL_MANAGER;

struct TunnelSnapshot {
    is_running: bool,
    connected_at: Option<u64>,
    instance: Option<ConnectedInstance>,
    metrics: Arc<RwLock<TunnelMetrics>>,
}

pub async fn get_vpn_status() -> Result<VpnStatus> {
    let Some(snapshot) = take_tunnel_snapshot()? else {
        return Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
            connected_at: None,
            connection_error: None,
        });
    };

    Ok(VpnStatus {
        connected: snapshot.is_running,
        instance: snapshot.instance,
        metrics: Some(snapshot.metrics.read().await.clone()),
        connected_at: snapshot.connected_at,
        connection_error: None,
    })
}

fn take_tunnel_snapshot() -> Result<Option<TunnelSnapshot>> {
    let manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;

    Ok(manager.as_ref().map(|handle| TunnelSnapshot {
        is_running: !handle.task.is_finished(),
        connected_at: handle
            .connected_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_secs()),
        instance: handle.instance.clone(),
        metrics: handle.metrics.clone(),
    }))
}
