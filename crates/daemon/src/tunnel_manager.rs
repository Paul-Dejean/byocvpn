use std::sync::{Arc, Mutex};

use byocvpn_core::tunnel::TunnelMetrics;
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};

#[cfg(target_os = "macos")]
use crate::dns_macos::DomainNameSystemOverrideGuard;

pub struct TunnelHandle {
    pub shutdown: watch::Sender<()>,
    pub task: JoinHandle<()>,
    pub metrics: Arc<RwLock<TunnelMetrics>>,
    pub metrics_task: JoinHandle<()>,
    pub metrics_shutdown: watch::Sender<()>,
    #[cfg(target_os = "macos")]
    pub domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard>,

    // Connection details
    pub instance_id: Option<String>,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
}

pub static TUNNEL_MANAGER: Mutex<Option<TunnelHandle>> = Mutex::new(None);
