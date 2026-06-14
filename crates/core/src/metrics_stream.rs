use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex as StdMutex,
    },
    time::Duration,
};

use crate::{
    error::{Result, SystemError},
    ipc::IpcStream,
    tunnel::{ConnectedInstance, TunnelMetrics, VpnStatus},
};
use log::*;

static STREAM_STATE: StdMutex<Option<(ConnectedInstance, Option<u64>)>> = StdMutex::new(None);
static STREAM_GENERATION: AtomicU64 = AtomicU64::new(0);

pub async fn start<F>(
    socket_path: PathBuf,
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
    on_update: F,
) -> Result<()>
where
    F: Fn(VpnStatus) + Send + 'static,
{
    let generation = {
        let mut state = STREAM_STATE.lock().map_err(|error| {
            warn!("Failed to acquire metrics stream state lock: {error}");
            SystemError::MutexPoisoned(error.to_string())
        })?;

        if state.is_some() {
            return Ok(());
        }

        *state = Some((connected_instance, connected_at));
        STREAM_GENERATION.fetch_add(1, Ordering::SeqCst) + 1
    };

    tokio::spawn(async move {
        let mut stream = {
            let mut maybe_stream = None;
            for attempt in 1..=5 {
                match IpcStream::connect(&socket_path).await {
                    Ok(connected_stream) => {
                        info!("Connected to metrics socket on attempt {}", attempt);
                        maybe_stream = Some(connected_stream);
                        break;
                    }
                    Err(error) => {
                        debug!(
                            "Failed to connect to metrics socket (attempt {}): {}",
                            attempt, error
                        );
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
            match maybe_stream {
                Some(connected_stream) => connected_stream,
                None => {
                    warn!("Failed to connect to metrics socket after retries");
                    if let Ok(mut state) = STREAM_STATE.lock() {
                        *state = None;
                    }
                    return;
                }
            }
        };

        'reconnect: loop {
            let daemon_crashed = 'read: loop {
                let (current_instance, current_timestamp) = {
                    let state_guard = match STREAM_STATE.lock() {
                        Ok(guard) => guard,
                        Err(error) => {
                            warn!("Failed to acquire metrics stream state lock: {error}");
                            break 'read false;
                        }
                    };
                    match state_guard.as_ref() {
                        None => break 'read false,
                        Some((instance, timestamp)) => (instance.clone(), *timestamp),
                    }
                };

                match stream.read_message().await {
                    Ok(Some(line)) => {
                        if let Ok(metrics) = serde_json::from_str::<TunnelMetrics>(&line) {
                            on_update(VpnStatus {
                                connected: true,
                                instance: Some(current_instance),
                                metrics: Some(metrics),
                                connected_at: current_timestamp,
                                connection_error: None,
                            });
                        } else {
                            error!("Failed to parse metrics: {}", line);
                        }
                    }
                    Ok(None) => {
                        info!("Metrics stream ended (daemon likely restarting)");
                        break 'read true;
                    }
                    Err(error) => {
                        error!("Error reading from metrics socket: {}", error);
                        break 'read true;
                    }
                }
            };

            if !daemon_crashed {
                info!("Metrics stream stopped cleanly");
                if let Ok(mut state) = STREAM_STATE.lock() {
                    *state = None;
                }
                return;
            }

            info!("Metrics stream disconnected; waiting for daemon to restart...");
            on_update(VpnStatus {
                connected: false,
                instance: None,
                metrics: None,
                connected_at: None,
                connection_error: None,
            });

            loop {
                tokio::time::sleep(Duration::from_secs(3)).await;

                let is_cancelled = STREAM_STATE
                    .lock()
                    .ok()
                    .map_or(true, |guard| guard.is_none())
                    || STREAM_GENERATION.load(Ordering::SeqCst) != generation;

                if is_cancelled {
                    info!("Metrics stream reconnect cancelled (superseded or stopped)");
                    return;
                }

                match IpcStream::connect(&socket_path).await {
                    Ok(new_stream) => {
                        info!("Reconnected to metrics socket after daemon restart");
                        stream = new_stream;
                        continue 'reconnect;
                    }
                    Err(error) => {
                        debug!("Metrics socket reconnect attempt failed: {}", error);
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn stop() -> Result<()> {
    let mut state = STREAM_STATE.lock().map_err(|error| {
        warn!("Failed to acquire metrics stream state lock: {error}");
        SystemError::MutexPoisoned(error.to_string())
    })?;

    *state = None;
    Ok(())
}
