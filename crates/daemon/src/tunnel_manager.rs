use std::sync::{Arc, Mutex};

use byocvpn_core::tunnel::{ConnectedInstance, TunnelMetrics};
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};

#[cfg(target_os = "linux")]
use crate::routing::dns_linux::DomainNameSystemOverrideGuard;
#[cfg(target_os = "macos")]
use crate::routing::dns_macos::DomainNameSystemOverrideGuard;
#[cfg(windows)]
use crate::routing::dns_windows::DomainNameSystemOverrideGuard;

pub struct TunnelHandle {
    pub shutdown: watch::Sender<()>,
    pub task: JoinHandle<()>,
    pub metrics: Arc<RwLock<TunnelMetrics>>,
    pub metrics_task: JoinHandle<()>,
    pub metrics_shutdown: watch::Sender<()>,
    pub route_monitor_task: JoinHandle<()>,
    pub route_monitor_shutdown: watch::Sender<()>,
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    pub domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard>,

    pub instance: Option<ConnectedInstance>,
}

pub static TUNNEL_MANAGER: Mutex<Option<TunnelHandle>> = Mutex::new(None);
