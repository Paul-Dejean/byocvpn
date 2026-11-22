use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::{
        Mutex as StdMutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::{
    error::{InternalError, Result},
    ipc::{IpcStream, metrics_socket_path},
    tunnel::{ConnectedInstance, TunnelMetrics, VpnStatus},
};

type StreamState = Option<(ConnectedInstance, Option<u64>)>;

static STREAM_STATE: StdMutex<StreamState> = StdMutex::new(None);
static STREAM_GENERATION: AtomicU64 = AtomicU64::new(0);
const METRICS_PERSIST_INTERVAL: Duration = Duration::from_secs(60);
const CONNECT_ATTEMPTS: u32 = 5;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(500);
const RECONNECT_DELAY: Duration = Duration::from_secs(3);

pub fn start_with_persistence<F, G>(
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
    on_status_update: F,
    on_metrics_persist: G,
) -> Result<()>
where
    F: Fn(VpnStatus) + Send + Sync + 'static,
    G: Fn(u64, u64) + Send + Sync + 'static,
{
    debug!("starting metrics subscription");
    start(connected_instance, connected_at, move |vpn_status| {
        thread_local! {
            static LAST_PERSIST: Cell<Option<Instant>> = const { Cell::new(None) };
        }
        let should_persist = LAST_PERSIST.with(|last_persist| {
            let is_due = last_persist
                .get()
                .is_none_or(|instant| instant.elapsed() >= METRICS_PERSIST_INTERVAL);
            if is_due {
                last_persist.set(Some(Instant::now()));
            }
            is_due
        });

        if should_persist && let Some(metrics) = vpn_status.metrics.as_ref() {
            on_metrics_persist(metrics.bytes_sent, metrics.bytes_received);
        }
        on_status_update(vpn_status);
    })
}

pub fn start<F>(
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
    on_update: F,
) -> Result<()>
where
    F: Fn(VpnStatus) + Send + Sync + 'static,
{
    let generation = {
        let mut state = lock_stream_state()?;
        if state.is_some() {
            return Ok(());
        }
        *state = Some((connected_instance, connected_at));
        drop(state);
        STREAM_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
    };

    tokio::spawn(run_stream(metrics_socket_path(), generation, on_update));
    Ok(())
}

pub fn stop() -> Result<()> {
    *lock_stream_state()? = None;
    Ok(())
}

async fn run_stream<F>(socket_path: PathBuf, generation: u64, on_update: F)
where
    F: Fn(VpnStatus) + Send + Sync + 'static,
{
    let Some(mut stream) = connect_with_retries(&socket_path).await else {
        warn!("failed to connect to metrics socket after retries");
        clear_stream_state();
        return;
    };

    loop {
        let daemon_crashed = forward_metrics(&mut stream, &on_update).await;
        if !daemon_crashed {
            debug!("metrics stream stopped cleanly");
            clear_stream_state();
            return;
        }

        warn!("metrics stream disconnected, waiting for the daemon to restart");
        on_update(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
            connected_at: None,
            connection_error: None,
        });

        match reconnect(&socket_path, generation).await {
            Some(new_stream) => stream = new_stream,
            None => return,
        }
    }
}

async fn connect_with_retries(socket_path: &Path) -> Option<IpcStream> {
    for attempt in 1..=CONNECT_ATTEMPTS {
        match IpcStream::connect(socket_path).await {
            Ok(stream) => {
                debug!("connected to metrics socket on attempt {attempt}");
                return Some(stream);
            }
            Err(error) => {
                debug!(%error, "failed to connect to metrics socket (attempt {attempt})");
                sleep(CONNECT_RETRY_DELAY).await;
            }
        }
    }
    None
}

async fn forward_metrics<F>(stream: &mut IpcStream, on_update: &F) -> bool
where
    F: Fn(VpnStatus) + Sync,
{
    loop {
        let Some((current_instance, current_timestamp)) = read_connected_instance() else {
            return false;
        };

        match stream.read_message().await {
            Ok(Some(line)) => match serde_json::from_str::<TunnelMetrics>(&line) {
                Ok(metrics) => on_update(VpnStatus {
                    connected: true,
                    instance: Some(current_instance),
                    metrics: Some(metrics),
                    connected_at: current_timestamp,
                    connection_error: None,
                }),
                Err(parse_error) => warn!(%parse_error, "failed to parse metrics {line}"),
            },
            Ok(None) => {
                debug!("metrics stream ended (daemon likely restarting)");
                return true;
            }
            Err(error) => {
                warn!(%error, "error reading from metrics socket");
                return true;
            }
        }
    }
}

async fn reconnect(socket_path: &Path, generation: u64) -> Option<IpcStream> {
    loop {
        sleep(RECONNECT_DELAY).await;

        let is_cancelled = STREAM_STATE.lock().ok().is_none_or(|guard| guard.is_none())
            || STREAM_GENERATION.load(Ordering::SeqCst) != generation;
        if is_cancelled {
            debug!("metrics stream reconnect cancelled (superseded or stopped)");
            return None;
        }

        match IpcStream::connect(socket_path).await {
            Ok(new_stream) => {
                info!("reconnected to metrics socket after daemon restart");
                return Some(new_stream);
            }
            Err(error) => debug!(%error, "metrics socket reconnect attempt failed"),
        }
    }
}

fn read_connected_instance() -> Option<(ConnectedInstance, Option<u64>)> {
    let state_guard = match STREAM_STATE.lock() {
        Ok(guard) => guard,
        Err(error) => {
            warn!(%error, "failed to acquire metrics stream state lock");
            return None;
        }
    };
    state_guard
        .as_ref()
        .map(|(instance, timestamp)| (instance.clone(), *timestamp))
}

fn clear_stream_state() {
    if let Ok(mut state) = STREAM_STATE.lock() {
        *state = None;
    }
}

#[expect(
    clippy::map_err_ignore,
    reason = "a PoisonError carries the guard, not a cause worth chaining"
)]
fn lock_stream_state() -> Result<MutexGuard<'static, StreamState>> {
    STREAM_STATE.lock().map_err(|_| {
        InternalError::MutexPoisoned {
            lock_name: "STREAM_STATE",
        }
        .into()
    })
}
