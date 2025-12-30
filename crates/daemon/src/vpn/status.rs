use byocvpn_core::{
    error::{Error, Result},
    tunnel::VpnStatus,
};

use crate::tunnel_manager::TUNNEL_MANAGER;

pub async fn get_vpn_status() -> Result<VpnStatus> {
    let manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| Error::TunnelCreationError("Mutex poisoned".to_string()))?;

    if let Some(handle) = manager.as_ref() {
        let is_running = !handle.task.is_finished();

        Ok(VpnStatus {
            connected: is_running,
            instance_id: handle.instance_id.clone(),
            public_ip_v4: handle.public_ip_v4.clone(),
            public_ip_v6: handle.public_ip_v6.clone(),
        })
    } else {
        Ok(VpnStatus {
            connected: false,
            instance_id: None,
            public_ip_v4: None,
            public_ip_v6: None,
        })
    }
}
