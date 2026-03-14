use std::{path::PathBuf, sync::Mutex as StdMutex, thread::sleep, time::Duration};

use crate::{
    error::{Result, SystemError},
    ipc::IpcStream,
    tunnel::{ConnectedInstance, TunnelMetrics, VpnStatus},
};
use log::*;

static STREAM_STATE: StdMutex<Option<ConnectedInstance>> = StdMutex::new(None);

pub async fn start<F>(
    socket_path: PathBuf,
    connected_instance: ConnectedInstance,
    on_update: F,
) -> Result<()>
where
    F: Fn(VpnStatus) + Send + 'static,
{
    {
        let mut state = STREAM_STATE.lock().map_err(|error| {
            error!("Failed to acquire metrics stream state lock: {error}");
            SystemError::MutexPoisoned(error.to_string())
        })?;

        if state.is_some() {
            return Ok(());
        }

        *state = Some(connected_instance);
    }

    tokio::spawn(async move {
        let mut stream = None;
        for attempt in 1..=5 {
            match IpcStream::connect(&socket_path).await {
                Ok(connected_stream) => {
                    info!("Connected to metrics socket on attempt {}", attempt);
                    stream = Some(connected_stream);
                    break;
                }
                Err(error) => {
                    error!(
                        "Failed to connect to metrics socket (attempt {}): {}",
                        attempt, error
                    );
                    sleep(Duration::from_millis(500));
                }
            }
        }

        let mut stream = match stream {
            Some(connected_stream) => connected_stream,
            None => {
                error!("Failed to connect to metrics socket after retries");
                if let Ok(mut state) = STREAM_STATE.lock() {
                    *state = None;
                }
                return;
            }
        };

        let mut stopped_cleanly = false;

        loop {
            let connected_instance = {
                let state_guard = match STREAM_STATE.lock() {
                    Ok(guard) => guard,
                    Err(error) => {
                        error!("Failed to acquire metrics stream state lock: {error}");
                        break;
                    }
                };
                match state_guard.as_ref() {
                    None => {
                        stopped_cleanly = true;
                        break;
                    }
                    Some(instance) => instance.clone(),
                }
            };

            match stream.read_message().await {
                Ok(Some(line)) => {
                    if let Ok(metrics) = serde_json::from_str::<TunnelMetrics>(&line) {
                        on_update(VpnStatus {
                            connected: true,
                            instance: Some(connected_instance),
                            metrics: Some(metrics),
                        });
                    } else {
                        error!("Failed to parse metrics: {}", line);
                    }
                }
                Ok(None) => {
                    info!("Metrics stream ended");
                    break;
                }
                Err(error) => {
                    error!("Error reading from metrics socket: {}", error);
                    break;
                }
            }
        }

        info!("Metrics stream stopped (clean={})", stopped_cleanly);

        if let Ok(mut state) = STREAM_STATE.lock() {
            *state = None;
        }

        if !stopped_cleanly {
            on_update(VpnStatus {
                connected: false,
                instance: None,
                metrics: None,
            });
        }
    });

    Ok(())
}

pub async fn stop() -> Result<()> {
    let mut state = STREAM_STATE.lock().map_err(|error| {
        error!("Failed to acquire metrics stream state lock: {error}");
        SystemError::MutexPoisoned(error.to_string())
    })?;

    *state = None;
    Ok(())
}
