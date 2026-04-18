use std::sync::{Arc, Mutex};

use byocvpn_core::tunnel::{ConnectedInstance, TunnelMetrics};
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};

use crate::routing::dns::DnsOverrideGuard;

pub struct TunnelHandle {
    pub shutdown: watch::Sender<()>,
    pub task: JoinHandle<()>,
    pub metrics: Arc<RwLock<TunnelMetrics>>,
    pub metrics_task: JoinHandle<()>,
    pub metrics_shutdown: watch::Sender<()>,
    pub route_monitor_task: JoinHandle<()>,
    pub route_monitor_shutdown: watch::Sender<()>,
    pub dns_override_guard: Option<DnsOverrideGuard>,
    pub server_ip: String,
    pub instance: Option<ConnectedInstance>,
}

pub static TUNNEL_MANAGER: Mutex<Option<TunnelHandle>> = Mutex::new(None);
