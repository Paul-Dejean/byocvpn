use std::{str::FromStr, sync::Mutex as StdMutex};

use byocvpn_aws::{AwsProvider, AwsProviderConfig};
use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName},
    commands, credentials,
    error::{Error, Result},
    tunnel::TunnelMetricsWithRates,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

// Global state to track if broadcaster is running
static METRICS_BROADCASTER: StdMutex<Option<()>> = StdMutex::new(None);

async fn create_cloud_provider(
    cloud_provider_name: &str,
    region: Option<String>,
) -> Result<Box<dyn CloudProvider>> {
    // Get stored credentials
    let credentials = credentials::get_credentials().await?;

    // Create AWS provider config
    match cloud_provider_name {
        "aws" => {
            let config = AwsProviderConfig {
                region,
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(&config).await;
            Ok(Box::new(cloud_provider))
        }
        _ => {
            return Err(Error::InvalidCloudProviderName(
                cloud_provider_name.to_string(),
            ));
        }
    }
}

#[tauri::command]
pub async fn save_credentials(
    cloud_provider_name: String,
    access_key_id: String,
    secret_access_key: String,
) -> Result<()> {
    let cloud_provider = match CloudProviderName::from_str(&cloud_provider_name) {
        Ok(provider) => provider,
        Err(_) => return Err(Error::InvalidCloudProviderName(cloud_provider_name)),
    };
    credentials::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await?;
    Ok(())
}

#[tauri::command]
pub async fn verify_permissions() -> Result<Value> {
    let cloud_provider = create_cloud_provider("aws", None).await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(region: String) -> Result<Value> {
    // Get stored credentials
    let cloud_provider = create_cloud_provider("aws", Some(region.clone())).await?;

    // Generate keypair for this instance
    commands::setup::setup(&*cloud_provider).await?;

    commands::setup::enable_region(&*cloud_provider, &region).await?;

    let (instance_id, public_ip_v4, public_ip_v6) =
        commands::spawn::spawn_instance(&*cloud_provider).await?;

    // Return instance details
    Ok(json!({
        "instance_id": instance_id,
        "public_ip_v4": public_ip_v4,
        "public_ip_v6": public_ip_v6,
        "region": region,
    }))
}

#[tauri::command]
pub async fn terminate_instance(instance_id: String, region: String) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &instance_id).await?;
    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: String) -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    // Terminate the instance
    let instances = commands::list::list_instances(&*cloud_provider).await?;

    Ok(instances
        .into_iter()
        .map(|instance| {
            json!({
                "id": instance.id,
                "name": instance.name,
                "state": instance.state,
                "public_ip_v4": instance.public_ip_v4,
                "public_ip_v6": instance.public_ip_v6,
            })
        })
        .collect::<Vec<Value>>())
}

#[tauri::command]
pub async fn has_profile() -> Result<bool> {
    // Try to get credentials - if they exist and are valid, return true
    match credentials::get_credentials().await {
        Ok(credentials) => {
            // Check if credentials are not empty
            if !credentials.access_key.is_empty() && !credentials.secret_access_key.is_empty() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => {
            // Credentials file doesn't exist or can't be read
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_regions() -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider("aws", None).await?;

    let regions = commands::setup::get_regions(&*cloud_provider).await?;

    Ok(regions
        .into_iter()
        .map(|r| {
            json!({
                "name": r.name,
                "country": r.country,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn connect(instance_id: String, region: String, app_handle: AppHandle) -> Result<String> {
    let cloud_provider = create_cloud_provider("aws", Some(region)).await?;
    let daemon_client = UnixDaemonClient;
    println!("Connecting to instance {}", instance_id.clone());
    commands::connect::connect(&*cloud_provider, &daemon_client, instance_id.clone()).await?;

    // Subscribe to metrics broadcast and emit as Tauri events
    // Small delay to ensure daemon is ready
    println!("Starting metrics stream...");
    match start_metrics_stream(app_handle).await {
        Ok(_) => println!("Started metrics stream"),
        Err(e) => println!("Failed to start metrics stream: {}", e),
    }

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id.clone()
    ))
}

#[tauri::command]
pub async fn disconnect() -> Result<String> {
    // Stop metrics streaming
    stop_metrics_stream().await?;

    let daemon_client = UnixDaemonClient;
    commands::disconnect::disconnect(&daemon_client).await?;
    Ok("Disconnected successfully.".to_string())
}

async fn start_metrics_stream(app_handle: AppHandle) -> Result<()> {
    println!("Starting metrics stream...");
    let mut broadcaster = match METRICS_BROADCASTER.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to acquire metrics broadcaster lock : {e}");
            return Err(Error::InvalidCloudProviderConfig(
                "Failed to acquire metrics broadcaster lock".to_string(),
            ));
        }
    };

    // Don't start if already running
    if broadcaster.is_some() {
        return Ok(());
    }

    // Mark as running
    *broadcaster = Some(());
    drop(broadcaster); // Release lock before spawning

    // Get metrics socket path
    let metrics_socket_path = byocvpn_daemon::metrics_socket_path();

    // Spawn task to read from IPC socket and forward to Tauri events
    tauri::async_runtime::spawn(async move {
        use std::{thread::sleep, time::Duration};

        use byocvpn_core::ipc::IpcStream;

        // Retry connection a few times in case daemon is still setting up
        let mut stream = None;
        for attempt in 1..=5 {
            match IpcStream::connect(&metrics_socket_path).await {
                Ok(s) => {
                    println!("Connected to metrics socket on attempt {}", attempt);
                    stream = Some(s);
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to connect to metrics socket (attempt {}): {}",
                        attempt, e
                    );
                    sleep(Duration::from_millis(500));
                }
            }
        }

        let stream = match stream {
            Some(s) => s,
            None => {
                eprintln!("Failed to connect to metrics socket after retries");
                let mut broadcaster = METRICS_BROADCASTER.lock().unwrap();
                *broadcaster = None;
                return;
            }
        };

        let (read, _write) = stream.into_split();
        let mut reader = read.into_buf_reader();

        loop {
            // Check if we should stop
            {
                let broadcaster = METRICS_BROADCASTER.lock().unwrap();
                if broadcaster.is_none() {
                    break;
                }
            }

            // Read metrics from IPC socket
            match reader.read_message().await {
                Ok(Some(line)) => {
                    // Parse JSON metrics
                    if let Ok(metrics) = serde_json::from_str::<TunnelMetricsWithRates>(&line) {
                        // Emit as Tauri event
                        let _ = app_handle.emit("vpn-metrics", &metrics);
                    } else {
                        eprintln!("Failed to parse metrics: {}", line);
                    }
                }
                Ok(None) => {
                    // Stream ended
                    println!("Metrics stream ended");
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading from metrics socket: {}", e);
                    break;
                }
            }
        }

        println!("Metrics forwarder stopped");

        // Clear the broadcaster flag
        let mut broadcaster = METRICS_BROADCASTER.lock().unwrap();
        *broadcaster = None;
    });

    Ok(())
}

async fn stop_metrics_stream() -> Result<()> {
    let mut broadcaster = METRICS_BROADCASTER.lock().unwrap();
    *broadcaster = None;
    Ok(())
}
