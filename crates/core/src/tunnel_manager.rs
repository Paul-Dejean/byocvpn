use std::sync::Mutex;

use once_cell::sync::Lazy;
use tokio::{sync::watch, task::JoinHandle};

pub struct TunnelHandle {
    pub shutdown: watch::Sender<()>,
    pub task: JoinHandle<()>,
}

pub static TUNNEL_MANAGER: Lazy<Mutex<Option<TunnelHandle>>> = Lazy::new(|| Mutex::new(None));
