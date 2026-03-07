use std::{str::FromStr, time::Instant};

use byocvpn_aws::{AwsProvider, AwsProviderConfig, pricing as aws_pricing};
use byocvpn_azure::{AzureProvider, AzureProviderConfig, pricing as azure_pricing};
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, PricingInfo, SpawnCompleteEvent, SpawnJob,
        SpawnProgressEvent, SpawnStepStatus,
    },
    commands, connectivity, credentials,
    crypto::generate_keypair,
    daemon_client::{DaemonClient, DaemonCommand},
    error::{ConfigurationError, Result},
    ledger::LedgerEntry,
    metrics_stream,
    tunnel::VpnStatus,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use byocvpn_gcp::{GcpProvider, GcpProviderConfig, pricing as gcp_pricing};
use byocvpn_oracle::pricing as oracle_pricing;
use chrono::Utc;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;

async fn create_cloud_provider(cloud_provider_name: &str) -> Result<Box<dyn CloudProvider>> {
    match cloud_provider_name {
        "aws" => {
            let credentials = credentials::get_credentials().await?;
            let config = AwsProviderConfig {
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(config).await;
            Ok(Box::new(cloud_provider) as Box<dyn CloudProvider>)
        }
        "oracle" => {
            let oracle_credentials = credentials::get_oracle_credentials().await?;
            let config = byocvpn_oracle::OracleProviderConfig {
                tenancy_ocid: oracle_credentials.tenancy_ocid,
                user_ocid: oracle_credentials.user_ocid,
                fingerprint: oracle_credentials.fingerprint,
                private_key_pem: oracle_credentials.private_key_pem,
                region: oracle_credentials.region,
            };
            Ok(Box::new(byocvpn_oracle::OracleProvider::new(config)) as Box<dyn CloudProvider>)
        }
        "gcp" => {
            let gcp_credentials = credentials::get_gcp_credentials().await?;
            let config = GcpProviderConfig {
                service_account_json: gcp_credentials.service_account_json,
            };
            Ok(Box::new(GcpProvider::new(config)?) as Box<dyn CloudProvider>)
        }
        "azure" => {
            let azure_credentials = credentials::get_azure_credentials().await?;
            let config = AzureProviderConfig {
                subscription_id: azure_credentials.subscription_id,
                tenant_id: azure_credentials.tenant_id,
                client_id: azure_credentials.client_id,
                client_secret: azure_credentials.client_secret,
            };
            let provider = AzureProvider::new(config)?;
            Ok(Box::new(provider) as Box<dyn CloudProvider>)
        }
        _ => Err(ConfigurationError::InvalidCloudProvider(cloud_provider_name.to_string()).into()),
    }
}

#[tauri::command]
pub async fn get_credentials(provider: String) -> Result<Value> {
    match provider.as_str() {
        "aws" => match credentials::get_credentials().await {
            Ok(creds) => Ok(json!({
                "accessKeyId": creds.access_key,
                "secretAccessKey": creds.secret_access_key,
            })),
            Err(_) => Ok(json!(null)),
        },
        "oracle" => match credentials::get_oracle_credentials().await {
            Ok(creds) => Ok(json!({
                "tenancyOcid": creds.tenancy_ocid,
                "userOcid": creds.user_ocid,
                "fingerprint": creds.fingerprint,
                "privateKeyPem": creds.private_key_pem,
                "region": creds.region,
            })),
            Err(_) => Ok(json!(null)),
        },
        "gcp" => match credentials::get_gcp_credentials().await {
            Ok(creds) => Ok(json!({
                "projectId": creds.project_id,
                "serviceAccountJson": creds.service_account_json,
            })),
            Err(_) => Ok(json!(null)),
        },
        "azure" => match credentials::get_azure_credentials().await {
            Ok(creds) => Ok(json!({
                "subscriptionId": creds.subscription_id,
                "tenantId": creds.tenant_id,
                "clientId": creds.client_id,
                "clientSecret": creds.client_secret,
            })),
            Err(_) => Ok(json!(null)),
        },
        _ => Err(ConfigurationError::InvalidCloudProvider(provider).into()),
    }
}

#[tauri::command]
pub async fn save_credentials(provider: String, creds: Value) -> Result<()> {
    match provider.as_str() {
        "aws" => {
            let access_key_id = creds["accessKeyId"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing accessKeyId".into())
                })?
                .to_string();
            let secret_access_key = creds["secretAccessKey"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing secretAccessKey".into())
                })?
                .to_string();
            let cloud_provider = CloudProviderName::from_str(&provider)
                .map_err(|_| ConfigurationError::InvalidCloudProvider(provider.clone()))?;
            credentials::save_credentials(&cloud_provider, &access_key_id, &secret_access_key).await
        }
        "oracle" => {
            let tenancy_ocid = creds["tenancyOcid"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing tenancyOcid".into())
                })?
                .to_string();
            let user_ocid = creds["userOcid"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing userOcid".into()))?
                .to_string();
            let fingerprint = creds["fingerprint"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing fingerprint".into())
                })?
                .to_string();
            let private_key_pem = creds["privateKeyPem"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing privateKeyPem".into())
                })?
                .to_string();
            let region = creds["region"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing region".into()))?
                .to_string();
            credentials::save_oracle_credentials(
                &tenancy_ocid,
                &user_ocid,
                &fingerprint,
                &private_key_pem,
                &region,
            )
            .await
        }
        "gcp" => {
            let project_id = creds["projectId"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing projectId".into())
                })?
                .to_string();
            let service_account_json = creds["serviceAccountJson"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing serviceAccountJson".into())
                })?
                .to_string();
            credentials::save_gcp_credentials(&project_id, &service_account_json).await
        }
        "azure" => {
            let subscription_id = creds["subscriptionId"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing subscriptionId".into())
                })?
                .to_string();
            let tenant_id = creds["tenantId"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing tenantId".into()))?
                .to_string();
            let client_id = creds["clientId"]
                .as_str()
                .ok_or_else(|| ConfigurationError::InvalidCloudProvider("missing clientId".into()))?
                .to_string();
            let client_secret = creds["clientSecret"]
                .as_str()
                .ok_or_else(|| {
                    ConfigurationError::InvalidCloudProvider("missing clientSecret".into())
                })?
                .to_string();
            credentials::save_azure_credentials(
                &subscription_id,
                &tenant_id,
                &client_id,
                &client_secret,
            )
            .await
        }
        _ => Err(ConfigurationError::InvalidCloudProvider(provider).into()),
    }
}

#[tauri::command]
pub async fn verify_permissions() -> Result<Value> {
    let cloud_provider = create_cloud_provider("aws").await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<SpawnJob> {
    let cloud_provider = create_cloud_provider(&provider).await?;

    // Generate WireGuard keypairs upfront so they can be moved into the task.
    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let steps = cloud_provider.spawn_steps(&region);
    let job_id = format!("{}-{}", provider, Utc::now().timestamp_millis());
    let job = SpawnJob {
        job_id: job_id.clone(),
        steps,
        region: region.clone(),
        provider: provider.clone(),
    };

    // Clone steps for the background task; `job` is returned to the caller.
    let steps_for_task = job.steps.clone();

    // Spawn background task so this command returns the SpawnJob immediately.
    // Progress is reported via three Tauri events:
    //   "spawn-progress" — SpawnProgressEvent on each step transition
    //   "spawn-complete" — SpawnCompleteEvent when the instance is fully ready
    //   "spawn-failed"   — { jobId, error } if any step fails
    tauri::async_runtime::spawn(async move {
        // Inline helper: emit a spawn-progress event for one step.
        // All borrows are short-lived and never cross an .await boundary.
        let emit_step = |step_id: &str, status: SpawnStepStatus, error: Option<String>| {
            let _ = app_handle.emit(
                "spawn-progress",
                SpawnProgressEvent {
                    job_id: job_id.clone(),
                    step_id: step_id.to_string(),
                    status,
                    error,
                },
            );
        };

        // Macro that marks a step failed, emits spawn-failed, and returns.
        macro_rules! fail {
            ($step_id:expr, $err:expr) => {{
                let msg = $err.to_string();
                emit_step($step_id, SpawnStepStatus::Failed, Some(msg.clone()));
                let _ = app_handle.emit("spawn-failed", json!({ "jobId": &job_id, "error": msg }));
                return;
            }};
        }

        // ── Execute deployment steps ──────────────────────────────────────
        // Reserved ids ("launch", "wireguard_ready") are handled inline;
        // everything else is dispatched to run_spawn_step.
        let mut spawned_instance: Option<InstanceInfo> = None;

        for step in &steps_for_task {
            match step.id.as_str() {
                "launch" => {
                    emit_step("launch", SpawnStepStatus::Running, None);
                    match commands::spawn::launch_instance(
                        &*cloud_provider,
                        &region,
                        &server_private_key,
                        &client_public_key,
                    )
                    .await
                    {
                        Ok(i) => {
                            emit_step("launch", SpawnStepStatus::Completed, None);
                            spawned_instance = Some(i);
                        }
                        Err(e) => {
                            fail!("launch", e);
                        }
                    }
                }
                "wireguard_ready" => {
                    let instance_ip = spawned_instance
                        .as_ref()
                        .expect("launch step must precede wireguard_ready")
                        .public_ip_v4
                        .clone();
                    emit_step("wireguard_ready", SpawnStepStatus::Running, None);
                    if let Err(e) = connectivity::wait_until_ready(&instance_ip).await {
                        fail!("wireguard_ready", e);
                    }
                    emit_step("wireguard_ready", SpawnStepStatus::Completed, None);
                }
                step_id_raw => {
                    let step_id = step_id_raw.to_string();
                    emit_step(&step_id, SpawnStepStatus::Running, None);
                    if let Err(e) = cloud_provider.run_spawn_step(&step_id, &region).await {
                        fail!(step_id.as_str(), e);
                    }
                    emit_step(&step_id, SpawnStepStatus::Completed, None);
                }
            }
        }

        let instance = spawned_instance.expect("launch step must have run");

        // ── write_config (silent — not a visible step) ────────────────────
        let provider_name = cloud_provider.get_provider_name();
        if let Err(e) = commands::spawn::write_wireguard_config(
            &provider_name,
            &region,
            &instance,
            &client_private_key,
            &server_public_key,
        )
        .await
        {
            let msg = e.to_string();
            let _ = app_handle.emit("spawn-failed", json!({ "jobId": &job_id, "error": msg }));
            return;
        }

        // ── ledger entry ──────────────────────────────────────────────────
        let entry = LedgerEntry {
            instance_id: instance.id.clone(),
            provider: provider.clone(),
            region: region.clone(),
            instance_type: instance.instance_type.clone(),
            launched_at: instance.launched_at.unwrap_or_else(Utc::now),
            terminated_at: None,
            bytes_sent: 0,
            bytes_received: 0,
        };
        if let Ok(store) = app_handle.store("ledger.json") {
            store.set(
                LedgerEntry::store_key(&instance.id),
                serde_json::to_value(&entry).unwrap_or_default(),
            );
            let _ = store.save();
        }

        // ── completion ────────────────────────────────────────────────────
        let _ = app_handle.emit(
            "spawn-complete",
            SpawnCompleteEvent {
                job_id: job_id.clone(),
                instance,
            },
        );
    });

    Ok(job)
}

#[tauri::command]
pub async fn terminate_instance(
    instance_id: String,
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(&provider).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &region, &instance_id).await?;

    // Mark the ledger entry as terminated.
    let store = app_handle
        .store("ledger.json")
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;
    let key = LedgerEntry::store_key(&instance_id);
    if let Some(mut entry_value) = store.get(&key) {
        if let Some(obj) = entry_value.as_object_mut() {
            obj.insert("terminatedAt".to_string(), json!(Utc::now().to_rfc3339()));
            store.set(key, entry_value);
            let _ = store.save();
        }
    }

    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(region: Option<String>) -> Result<Vec<InstanceInfo>> {
    let mut all_instances: Vec<InstanceInfo> = Vec::new();

    for provider_name in &["aws", "oracle", "gcp", "azure"] {
        match create_cloud_provider(provider_name).await {
            Ok(provider) => {
                match commands::list::list_instances(&*provider, region.as_deref()).await {
                    Ok(instances) => all_instances.extend(instances),
                    Err(error) => {
                        eprintln!("Failed to list {} instances: {}", provider_name, error)
                    }
                }
            }
            Err(_) => {} // Provider not configured; skip silently.
        }
    }

    Ok(all_instances)
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
pub async fn get_regions(provider: String) -> Result<Vec<Value>> {
    let cloud_provider = create_cloud_provider(&provider).await?;

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

async fn fetch_vpn_status() -> Result<VpnStatus> {
    let daemon_client = UnixDaemonClient;

    if !daemon_client.is_daemon_running().await {
        return Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
        });
    }

    let response = daemon_client.send_command(DaemonCommand::Status).await?;

    serde_json::from_str(&response).map_err(|error| {
        ConfigurationError::InvalidCloudProvider(format!("Failed to parse status: {}", error))
            .into()
    })
}

#[tauri::command]
pub async fn connect(
    instance_id: String,
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(&provider).await?;
    let daemon_client = UnixDaemonClient;

    commands::connect::connect(
        &*cloud_provider,
        &daemon_client,
        region.as_str(),
        &instance_id,
    )
    .await?;

    let vpn_status = fetch_vpn_status().await?;

    if let Some(ref connected_instance) = vpn_status.instance {
        let emit_handle = app_handle.clone();
        if let Err(error) = metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            move |status| {
                let _ = emit_handle.emit("vpn-status", &status);
            },
        )
        .await
        {
            eprintln!("Failed to start metrics stream: {}", error);
        }
    }

    let _ = app_handle.emit("vpn-status", &vpn_status);

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id
    ))
}

#[tauri::command]
pub async fn disconnect(app_handle: AppHandle) -> Result<String> {
    metrics_stream::stop().await?;

    let daemon_client = UnixDaemonClient;
    commands::disconnect::disconnect(&daemon_client).await?;

    let _ = app_handle.emit(
        "vpn-status",
        &VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
        },
    );

    Ok("Disconnected successfully.".to_string())
}

#[tauri::command]
pub async fn get_vpn_status() -> Result<VpnStatus> {
    fetch_vpn_status().await
}

#[tauri::command]
pub async fn subscribe_to_vpn_status(app_handle: AppHandle) -> Result<()> {
    let status = fetch_vpn_status().await?;
    match status.instance {
        Some(connected_instance) => {
            let instance_id = connected_instance.instance_id.clone();
            metrics_stream::start(
                byocvpn_daemon::constants::metrics_socket_path(),
                connected_instance,
                move |vpn_status| {
                    let _ = app_handle.emit("vpn-status", &vpn_status);

                    // Throttle ledger writes to once per 60 seconds.
                    // last_write is a thread-local so we don't need shared state.
                    use std::cell::Cell;
                    thread_local! {
                        static LAST_WRITE: Cell<Option<Instant>> = const { Cell::new(None) };
                    }
                    let should_write = LAST_WRITE.with(|last| {
                        let now = Instant::now();
                        let write = last.get().map_or(true, |t| t.elapsed().as_secs() >= 60);
                        if write {
                            last.set(Some(now));
                        }
                        write
                    });

                    if should_write {
                        if let Some(ref metrics) = vpn_status.metrics {
                            let store = match app_handle.store("ledger.json") {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            let key = LedgerEntry::store_key(&instance_id);
                            if let Some(mut entry_value) = store.get(&key) {
                                if let Some(obj) = entry_value.as_object_mut() {
                                    obj.insert("bytesSent".to_string(), json!(metrics.bytes_sent));
                                    obj.insert(
                                        "bytesReceived".to_string(),
                                        json!(metrics.bytes_received),
                                    );
                                    store.set(key, entry_value);
                                    let _ = store.save();
                                }
                            }
                        }
                    }
                },
            )
            .await
        }
        None => Err(ConfigurationError::InvalidCloudProvider(
            "Cannot subscribe to VPN status: not connected to VPN".to_string(),
        )
        .into()),
    }
}

#[tauri::command]
pub async fn get_instance_pricing(provider: String, instance_type: String) -> Result<PricingInfo> {
    let pricing = match provider.as_str() {
        "aws" => aws_pricing::get_pricing(&instance_type),
        "azure" => azure_pricing::get_pricing(&instance_type),
        "gcp" => gcp_pricing::get_pricing(&instance_type),
        "oracle" => oracle_pricing::get_pricing(&instance_type),
        _ => None,
    };
    pricing.ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider(format!(
            "No pricing data for {}/{}",
            provider, instance_type
        ))
        .into()
    })
}

#[tauri::command]
pub async fn get_ledger(app_handle: AppHandle) -> Result<Vec<Value>> {
    let store = app_handle
        .store("ledger.json")
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;
    let entries: Vec<Value> = store
        .keys()
        .into_iter()
        .filter(|key| key.starts_with("ledger/"))
        .filter_map(|key| store.get(&key))
        .collect();
    Ok(entries)
}

#[tauri::command]
pub async fn is_daemon_installed() -> Result<bool> {
    let installed_binary_name = if cfg!(debug_assertions) {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let label = if cfg!(debug_assertions) {
        "com.byocvpn.daemon.dev"
    } else {
        "com.byocvpn.daemon"
    };
    Ok(std::path::Path::new(&format!(
        "/Library/PrivilegedHelperTools/{}",
        installed_binary_name
    ))
    .exists()
        && std::path::Path::new(&format!("/Library/LaunchDaemons/{}.plist", label)).exists())
}

#[tauri::command]
pub async fn install_daemon() -> Result<()> {
    // In debug mode install the dev daemon; in release install the production daemon.
    let is_dev = cfg!(debug_assertions);

    let installed_binary_name = if is_dev {
        "byocvpn-daemon-dev"
    } else {
        "byocvpn-daemon"
    };
    let plist_name = if is_dev {
        "com.byocvpn.daemon.dev.plist"
    } else {
        "com.byocvpn.daemon.plist"
    };
    let label = if is_dev {
        "com.byocvpn.daemon.dev"
    } else {
        "com.byocvpn.daemon"
    };
    let build_dir = if is_dev { "debug" } else { "release" };

    let current_executable_path = std::env::current_exe()
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;

    let workspace_root = current_executable_path
        .ancestors()
        .find(|path| path.join("Cargo.toml").exists());

    let exe_dir = current_executable_path.parent().ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider("Could not determine exe directory".to_string())
    })?;

    // Look for the daemon binary: bundled Resources/ first, then alongside exe, then workspace target/
    let daemon_binary_path = [
        exe_dir
            .parent()
            .map(|p| p.join("Resources").join("byocvpn-daemon"))
            .unwrap_or_default(),
        exe_dir.join(installed_binary_name),
        exe_dir.join("byocvpn_daemon"),
        workspace_root
            .map(|root| root.join("target").join(build_dir).join("byocvpn_daemon"))
            .unwrap_or_default(),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or_else(|| {
        ConfigurationError::InvalidCloudProvider(format!(
            "Daemon binary not found in target/{}/",
            build_dir
        ))
    })?;

    let daemon_plist_path = [
        exe_dir
            .parent()
            .map(|p| p.join("Resources").join(plist_name))
            .unwrap_or_default(),
        workspace_root
            .map(|root| root.join("scripts").join(plist_name))
            .unwrap_or_default(),
    ]
    .into_iter()
    .find(|path| path.exists())
    .ok_or_else(|| ConfigurationError::InvalidCloudProvider(format!("{} not found", plist_name)))?;

    let script = format!(
        r#"do shell script "
            launchctl unload '/Library/LaunchDaemons/{label}.plist' 2>/dev/null; \
            cp '{}' '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            chmod 544 '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            chown root:wheel '/Library/PrivilegedHelperTools/{installed_binary_name}' && \
            cp '{}' '/Library/LaunchDaemons/{label}.plist' && \
            chmod 644 '/Library/LaunchDaemons/{label}.plist' && \
            chown root:wheel '/Library/LaunchDaemons/{label}.plist' && \
            launchctl load '/Library/LaunchDaemons/{label}.plist'
        " with administrator privileges"#,
        daemon_binary_path.display(),
        daemon_plist_path.display(),
        label = label,
        installed_binary_name = installed_binary_name,
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|error| ConfigurationError::InvalidCloudProvider(error.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        Err(ConfigurationError::InvalidCloudProvider(format!(
            "osascript failed: {}",
            detail.trim()
        ))
        .into())
    }
}
