use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::SystemTime,
};

use byocvpn_core::{
    error::{InternalError, Result},
    tunnel::{ConnectedInstance, TunnelMetrics},
};
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
    pub interface_index: u32,
    pub instance: Option<ConnectedInstance>,
    pub connected_at: SystemTime,
}

pub static TUNNEL_MANAGER: Mutex<Option<TunnelHandle>> = Mutex::new(None);

#[expect(
    clippy::map_err_ignore,
    reason = "a PoisonError carries the guard, not a cause worth chaining"
)]
pub fn lock_tunnel_manager() -> Result<MutexGuard<'static, Option<TunnelHandle>>> {
    TUNNEL_MANAGER.lock().map_err(|_| {
        InternalError::MutexPoisoned {
            lock_name: "TUNNEL_MANAGER",
        }
        .into()
    })
}
