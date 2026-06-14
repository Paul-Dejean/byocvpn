use byocvpn_core::{daemon_client::VpnConnectParams, wireguard_config::parse_wireguard_config};
use log::*;

use crate::vpn::{connect::connect_vpn, session};

pub async fn try_restore_session() {
    let persisted_session = match session::read_session() {
        Some(persisted_session) => persisted_session,
        None => return,
    };

    info!(
        "Found persisted session for instance {}, attempting to restore...",
        persisted_session.instance_id
    );

    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            warn!("Session restore: cannot determine home directory");
            clear_stale_session();
            return;
        }
    };

    let config_path = home_dir
        .join(".byocvpn")
        .join("configs")
        .join(format!("{}.conf", persisted_session.instance_id));

    if !config_path.exists() {
        warn!(
            "Session restore: WireGuard config not found at {}",
            config_path.display()
        );
        clear_stale_session();
        return;
    }

    let wireguard_config = match parse_wireguard_config(&config_path.to_string_lossy()).await {
        Ok(wireguard_config) => wireguard_config,
        Err(error) => {
            warn!("Session restore: failed to parse WireGuard config: {}", error);
            clear_stale_session();
            return;
        }
    };

    let restore_instance_id = wireguard_config.instance_id.clone();

    let params = VpnConnectParams {
        instance_id: wireguard_config.instance_id,
        private_key: wireguard_config.private_key,
        public_key: wireguard_config.public_key,
        server_endpoint: wireguard_config.server_endpoint,
        private_ipv4: wireguard_config.private_ipv4,
        private_ipv6: wireguard_config.private_ipv6,
        dns_servers: wireguard_config.dns_servers,
        region: persisted_session.region,
        provider: persisted_session.provider,
        public_ip_v4: persisted_session.public_ip_v4,
        public_ip_v6: persisted_session.public_ip_v6,
        kill_switch_enabled: persisted_session.kill_switch_enabled,
    };

    match connect_vpn(params).await {
        Ok(()) => info!("Session restored successfully for instance {}", restore_instance_id),
        Err(error) => {
            warn!(
                "Session restore failed for instance {}: {}",
                restore_instance_id, error
            );
            clear_stale_session();
        }
    }
}

fn clear_stale_session() {
    if let Err(error) = session::clear_session() {
        warn!("Failed to clear stale session file: {}", error);
    }
}
