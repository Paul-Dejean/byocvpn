use std::{cell::Cell, path::PathBuf, time::Instant};

use log::*;

use crate::{
    error::Result,
    metrics_stream,
    tunnel::{ConnectedInstance, VpnStatus},
};

pub async fn start_metrics_subscription<F, G>(
    socket_path: PathBuf,
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
    on_status_update: F,
    on_metrics_persist: G,
) -> Result<()>
where
    F: Fn(VpnStatus) + Send + 'static,
    G: Fn(u64, u64) + Send + 'static,
{
    info!(
        "Starting metrics subscription via socket: {}",
        socket_path.display()
    );
    metrics_stream::start(socket_path, connected_instance, connected_at, move |vpn_status| {
        thread_local! {
            static LAST_PERSIST: Cell<Option<Instant>> = const { Cell::new(None) };
        }
        let should_persist = LAST_PERSIST.with(|last| {
            let now = Instant::now();
            let due = last.get().map_or(true, |t| t.elapsed().as_secs() >= 60);
            if due {
                last.set(Some(now));
            }
            due
        });

        if should_persist {
            if let Some(ref metrics) = vpn_status.metrics {
                on_metrics_persist(metrics.bytes_sent, metrics.bytes_received);
            }
        }

        on_status_update(vpn_status);
    })
    .await
}
