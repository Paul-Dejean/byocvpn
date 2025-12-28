use std::sync::{Arc, Mutex};

use byocvpn_core::tunnel::TunnelMetrics;
use once_cell::sync::Lazy;
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};

pub struct TunnelHandle {
    pub shutdown: watch::Sender<()>,
    pub task: JoinHandle<()>,
    pub metrics: Arc<RwLock<TunnelMetrics>>,
    pub metrics_task: JoinHandle<()>,
    pub metrics_shutdown: watch::Sender<()>,
    #[cfg(target_os = "macos")]
    pub domain_name_system_override_guard: Option<crate::dns_macos::DomainNameSystemOverrideGuard>,
}

pub static TUNNEL_MANAGER: Lazy<Mutex<Option<TunnelHandle>>> = Lazy::new(|| Mutex::new(None));
