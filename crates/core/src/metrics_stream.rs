use std::{path::PathBuf, sync::Mutex as StdMutex, thread::sleep, time::Duration};

use crate::{
    error::{ConfigurationError, Result},
    ipc::IpcStream,
    tunnel::{ConnectedInstance, TunnelMetrics, VpnStatus},
};

/// Global state: tracks the connected instance while the metrics stream is
/// running. Setting this to `None` signals the background task to stop cleanly.
static STREAM_STATE: StdMutex<Option<ConnectedInstance>> = StdMutex::new(None);

/// Start the metrics stream in the background.
///
/// Connects to `socket_path`, reads `TunnelMetrics` messages, and calls
/// `on_update` with a full [`VpnStatus`] on each tick.
///
/// If the socket closes unexpectedly (i.e. the tunnel died without a call to
/// [`stop`]) `on_update` is called once more with a disconnected `VpnStatus`
/// so callers can react immediately.
///
/// Calling `start` while a stream is already running is a no-op.
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
            eprintln!("Failed to acquire metrics stream state lock: {error}");
            ConfigurationError::InvalidCloudProvider(
                "Failed to acquire metrics stream state lock".to_string(),
            )
        })?;

        // Already running – nothing to do.
        if state.is_some() {
            return Ok(());
        }

        *state = Some(connected_instance);
    }

    tokio::spawn(async move {
        // Retry connection a few times in case the daemon is still setting up.
        let mut stream = None;
        for attempt in 1..=5 {
            match IpcStream::connect(&socket_path).await {
                Ok(connected_stream) => {
                    println!("Connected to metrics socket on attempt {}", attempt);
                    stream = Some(connected_stream);
                    break;
                }
                Err(error) => {
                    eprintln!(
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
                eprintln!("Failed to connect to metrics socket after retries");
                if let Ok(mut state) = STREAM_STATE.lock() {
                    *state = None;
                }
                return;
            }
        };

        let mut stopped_cleanly = false;

        loop {
            // Check whether stop() has been called and grab the current
            // instance. The scoped block ensures the MutexGuard is dropped
            // before the async read_message() call below.
            let connected_instance = {
                let state_guard = match STREAM_STATE.lock() {
                    Ok(guard) => guard,
                    Err(error) => {
                        eprintln!("Failed to acquire metrics stream state lock: {error}");
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
                        eprintln!("Failed to parse metrics: {}", line);
                    }
                }
                Ok(None) => {
                    println!("Metrics stream ended");
                    break;
                }
                Err(error) => {
                    eprintln!("Error reading from metrics socket: {}", error);
                    break;
                }
            }
        }

        println!("Metrics stream stopped (clean={})", stopped_cleanly);

        if let Ok(mut state) = STREAM_STATE.lock() {
            *state = None;
        }

        // If the tunnel closed on its own (not triggered by a disconnect
        // command), emit one final disconnected status so callers react.
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

/// Stop the metrics stream gracefully. Signals the background task to exit on
/// its next loop iteration.
pub async fn stop() -> Result<()> {
    let mut state = STREAM_STATE.lock().map_err(|error| {
        eprintln!("Failed to acquire metrics stream state lock: {error}");
        ConfigurationError::InvalidCloudProvider("Failed to stop metrics stream".to_string())
    })?;

    *state = None;
    Ok(())
}
