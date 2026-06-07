use std::{collections::HashSet, str::FromStr};


use byocvpn_aws::{AwsProvider, pricing as aws_pricing};
use byocvpn_azure::{AzureProvider, pricing as azure_pricing};
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, EnableRegionCompleteEvent, EnableRegionJob,
        EnableRegionProgressEvent, InstanceInfo, InstanceState, PricingInfo,
        ProvisionAccountCompleteEvent, ProvisionAccountJob, ProvisionAccountProgressEvent,
        SpawnCompleteEvent, SpawnJob, SpawnProgressEvent,
    },
    commands,
    commands::setup::Region,
    connectivity::{self, ProbeStatus},
    credentials::CredentialStore,
    crypto::generate_keypair,
    daemon_client::{DaemonClient, DaemonCommand},
    error::{Error, Result},
    ledger::LedgerEntry,
    metrics_stream,
    tunnel::VpnStatus,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use byocvpn_gcp::{GcpProvider, pricing as gcp_pricing};
use byocvpn_oracle::pricing as oracle_pricing;
use chrono::Utc;
use log::*;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use crate::ledger_store::LedgerStore;
use crate::provider_credentials::ProviderCredentials;
use crate::provider_store::ProviderStore;
use crate::settings_store::SettingsStore;
use crate::spawn_job_registry::{ActiveSpawnJob, SpawnJobRegistry};
use crate::tray;

async fn create_cloud_provider(provider_name: CloudProviderName) -> Result<Box<dyn CloudProvider>> {
    debug!("Creating {} cloud provider", provider_name);
    let store = CredentialStore::load().await?;
    let provider: Box<dyn CloudProvider> = match provider_name {
        CloudProviderName::Aws => {
            Box::new(AwsProvider::new(AwsCredentials::from_store(&store)?.into()).await)
        }
        CloudProviderName::Oracle => Box::new(byocvpn_oracle::OracleProvider::new(
            OracleCredentials::from_store(&store)?.into(),
        )),
        CloudProviderName::Gcp => Box::new(GcpProvider::new(
            GcpCredentials::from_store(&store)?.into(),
        )?),
        CloudProviderName::Azure => Box::new(AzureProvider::new(
            AzureCredentials::from_store(&store)?.into(),
        )?),
    };
    Ok(provider)
}

#[tauri::command]
pub async fn get_credentials(provider: String) -> Result<Option<ProviderCredentials>> {
    let store = match CredentialStore::load().await {
        Ok(store) => store,
        Err(_) => return Ok(None),
    };
    let provider_name = CloudProviderName::from_str(&provider)?;
    Ok(ProviderCredentials::load(provider_name, &store).ok())
}

#[tauri::command]
pub async fn save_credentials(credentials: ProviderCredentials) -> Result<()> {
    let mut store = CredentialStore::load().await?;
    credentials.write_to_store(&mut store);
    store.save()
}

#[tauri::command]
pub async fn delete_credentials(provider: String, app_handle: AppHandle) -> Result<()> {
    let mut store = CredentialStore::load().await?;
    let provider_name = CloudProviderName::from_str(&provider)?;
    let section = match provider_name {
        CloudProviderName::Aws => "AWS",
        CloudProviderName::Oracle => "ORACLE",
        CloudProviderName::Gcp => "GCP",
        CloudProviderName::Azure => "AZURE",
    };
    store.delete_section(section);
    store.save()?;
    if let Some(provider_store) = ProviderStore::open(&app_handle) {
        provider_store.clear_provisioned(&provider_name.to_string());
    } else {
        debug!(
            "Provider store unavailable when deleting credentials for {}",
            provider_name
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn verify_permissions() -> Result<Value> {
    let cloud_provider = create_cloud_provider(CloudProviderName::Aws).await?;
    let result = commands::verify_permissions::verify_permissions(&*cloud_provider).await;
    return result;
}

#[tauri::command]
pub async fn spawn_instance(
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<SpawnJob> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name.clone()).await?;

    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let job = SpawnJob {
        job_id: format!("{}-{}", provider_name, Utc::now().timestamp_millis()),
        steps: cloud_provider.get_spawn_steps(&region),
        region: region.clone(),
        provider: provider_name.clone(),
    };

    let job_id = job.job_id.clone();
    let steps = job.steps.clone();

    app_handle.state::<SpawnJobRegistry>().register(job.clone());

    tauri::async_runtime::spawn(async move {
        let job_id_for_progress = job_id.clone();
        let job_id_for_launched = job_id.clone();
        let progress_handle = app_handle.clone();
        let launched_handle = app_handle.clone();
        let region_for_launched = region.clone();
        let provider_name_for_launched = provider_name.clone();

        let result = commands::spawn::run_spawn_steps(
            &*cloud_provider,
            &steps,
            &region,
            &client_private_key,
            &server_private_key,
            &client_public_key,
            &server_public_key,
            move |step_id, status, error| {
                progress_handle
                    .state::<SpawnJobRegistry>()
                    .update_step_status(&job_id_for_progress, step_id, status.clone());

                if let Err(error) = progress_handle.emit(
                    "spawn-progress",
                    SpawnProgressEvent {
                        job_id: job_id_for_progress.clone(),
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                ) {
                    warn!("Failed to emit spawn-progress: {}", error);
                }
            },
            move |instance| {
                launched_handle
                    .state::<SpawnJobRegistry>()
                    .set_instance_id(&job_id_for_launched, instance.id.clone());

                if let Some(ledger) = LedgerStore::open(&launched_handle) {
                    let entry = LedgerEntry {
                        instance_id: instance.id.clone(),
                        provider: provider_name_for_launched.clone(),
                        region: region_for_launched.clone(),
                        instance_type: instance.instance_type.clone(),
                        launched_at: instance.launched_at.unwrap_or_else(Utc::now),
                        terminated_at: None,
                        setup_complete: false,
                        bytes_sent: 0,
                        bytes_received: 0,
                    };
                    ledger.set_entry(&entry);
                }

                let mut installing_instance = instance.clone();
                installing_instance.state = InstanceState::Installing;
                if let Err(error) = launched_handle.emit(
                    "spawn-instance-launched",
                    json!({ "jobId": &job_id_for_launched, "instance": installing_instance }),
                ) {
                    warn!("Failed to emit spawn-instance-launched: {}", error);
                }
            },
        )
        .await;

        app_handle.state::<SpawnJobRegistry>().deregister(&job_id);

        match result {
            Ok(mut instance) => {
                if let Some(ledger) = LedgerStore::open(&app_handle) {
                    ledger.mark_setup_complete(&instance.id);
                }
                instance.state = InstanceState::Running;
                if let Err(error) =
                    app_handle.emit("spawn-complete", SpawnCompleteEvent { job_id, instance })
                {
                    warn!("Failed to emit spawn-complete: {}", error);
                }
            }
            Err(error) => {
                if let Err(error) = app_handle.emit(
                    "spawn-failed",
                    json!({ "jobId": &job_id, "error": error.to_string() }),
                ) {
                    warn!("Failed to emit spawn-failed: {}", error);
                }
            }
        }
    });

    Ok(job)
}

#[tauri::command]
pub async fn list_active_spawn_jobs(
    app_handle: AppHandle,
) -> Result<Vec<ActiveSpawnJob>> {
    Ok(app_handle.state::<SpawnJobRegistry>().list())
}

#[tauri::command]
pub async fn terminate_instance(
    instance_id: String,
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<String> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &region, &instance_id).await?;

    if let Some(ledger) = LedgerStore::open(&app_handle) {
        ledger.mark_terminated(&instance_id);
    }

    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
pub async fn list_instances(
    region: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<InstanceInfo>> {
    let region_ref = region.as_deref();

    async fn list_provider_instances(
        provider_name: CloudProviderName,
        region: Option<&str>,
    ) -> (CloudProviderName, Option<Vec<InstanceInfo>>) {
        match create_cloud_provider(provider_name.clone()).await {
            Ok(provider) => match commands::list::list_instances(&*provider, region).await {
                Ok(instances) => (provider_name, Some(instances)),
                Err(error) => {
                    error!("Failed to list {} instances: {}", provider_name, error);
                    (provider_name, None)
                }
            },
            Err(error) => {
                debug!("No credentials for {}, skipping: {}", provider_name, error);
                (provider_name, None)
            }
        }
    }

    let (r_aws, r_oracle, r_gcp, r_azure) = tokio::join!(
        list_provider_instances(CloudProviderName::Aws, region_ref),
        list_provider_instances(CloudProviderName::Oracle, region_ref),
        list_provider_instances(CloudProviderName::Gcp, region_ref),
        list_provider_instances(CloudProviderName::Azure, region_ref),
    );

    let mut all_instances: Vec<InstanceInfo> = Vec::new();
    let mut queried_provider_names: Vec<CloudProviderName> = Vec::new();
    for (provider_name, result) in [r_aws, r_oracle, r_gcp, r_azure] {
        if let Some(instances) = result {
            queried_provider_names.push(provider_name);
            all_instances.extend(instances);
        }
    }

    if let Some(ledger) = LedgerStore::open(&app_handle) {
        let running_ids: HashSet<&str> = all_instances.iter().map(|i| i.id.as_str()).collect();
        ledger.reconcile_terminated(&running_ids, &queried_provider_names);

        let in_progress_ids = app_handle.state::<SpawnJobRegistry>().instance_ids_in_progress();

        let mut probe_handles = Vec::new();
        for instance in &all_instances {
            if instance.state != InstanceState::Running {
                continue;
            }
            if in_progress_ids.contains(&instance.id) {
                continue;
            }
            let instance_id = instance.id.clone();
            let instance_ip = instance.public_ip_v4.clone();
            probe_handles.push(tokio::spawn(async move {
                (instance_id, connectivity::probe_status(&instance_ip).await)
            }));
        }

        let mut probe_results = std::collections::HashMap::new();
        for handle in probe_handles {
            if let Ok((instance_id, status)) = handle.await {
                probe_results.insert(instance_id, status);
            }
        }

        for instance in &mut all_instances {
            if in_progress_ids.contains(&instance.id) {
                instance.state = InstanceState::Installing;
            } else if let Some(probe_status) = probe_results.remove(&instance.id) {
                match probe_status {
                    ProbeStatus::Ready => {
                        ledger.mark_setup_complete(&instance.id);
                        instance.state = InstanceState::Running;
                    }
                    ProbeStatus::Error(reason) => {
                        instance.state = InstanceState::Error;
                        instance.error_reason = Some(reason);
                    }
                    ProbeStatus::Installing => {
                        instance.state = InstanceState::Installing;
                    }
                }
            }
        }
    }

    Ok(all_instances)
}

#[tauri::command]
pub async fn has_profile() -> Result<bool> {
    let store = match CredentialStore::load().await {
        Ok(store) => store,
        Err(_) => return Ok(false),
    };
    Ok(AwsCredentials::from_store(&store).is_ok()
        || OracleCredentials::from_store(&store).is_ok()
        || GcpCredentials::from_store(&store).is_ok()
        || AzureCredentials::from_store(&store).is_ok())
}

#[tauri::command]
pub async fn provision_account(
    provider: String,
    app_handle: AppHandle,
) -> Result<ProvisionAccountJob> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name.clone()).await?;

    let job = ProvisionAccountJob {
        job_id: format!("{}-{}", provider_name, Utc::now().timestamp_millis()),
        steps: cloud_provider.get_provision_account_steps(),
        provider: provider_name,
    };

    let job_id = job.job_id.clone();
    let steps = job.steps.clone();

    tauri::async_runtime::spawn(async move {
        let job_id_for_progress = job_id.clone();
        let progress_handle = app_handle.clone();

        let result = commands::setup::run_provision_account_steps(
            &*cloud_provider,
            &steps,
            move |step_id, status, error| {
                if let Err(error) = progress_handle.emit(
                    "provision-account-progress",
                    ProvisionAccountProgressEvent {
                        job_id: job_id_for_progress.clone(),
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                ) {
                    warn!("Failed to emit provision-account-progress: {}", error);
                }
            },
        )
        .await;

        match result {
            Ok(()) => {
                if let Some(provider_store) =
                    ProviderStore::open(&app_handle)
                {
                    provider_store.mark_provisioned(&provider);
                } else {
                    debug!(
                        "Provider store unavailable when marking {} provisioned",
                        provider
                    );
                }
                if let Err(error) = app_handle.emit(
                    "provision-account-complete",
                    ProvisionAccountCompleteEvent {
                        job_id,
                        provider: cloud_provider.get_provider_name(),
                    },
                ) {
                    warn!("Failed to emit provision-account-complete: {}", error);
                }
            }
            Err(error) => {
                if let Err(error) = app_handle.emit(
                    "provision-account-failed",
                    json!({ "jobId": &job_id, "error": error.to_string() }),
                ) {
                    warn!("Failed to emit provision-account-failed: {}", error);
                }
            }
        }
    });

    Ok(job)
}

#[tauri::command]
pub async fn enable_region(
    region: String,
    provider: String,
    app_handle: AppHandle,
) -> Result<EnableRegionJob> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name.clone()).await?;

    let job = EnableRegionJob {
        job_id: format!(
            "{}-{}-{}",
            provider_name,
            region,
            Utc::now().timestamp_millis()
        ),
        steps: cloud_provider.get_enable_region_steps(&region),
        region: region.clone(),
        provider: provider_name,
    };

    let job_id = job.job_id.clone();
    let steps = job.steps.clone();

    tauri::async_runtime::spawn(async move {
        let job_id_for_progress = job_id.clone();
        let progress_handle = app_handle.clone();

        let result = commands::setup::run_enable_region_steps(
            &*cloud_provider,
            &steps,
            &region,
            move |step_id, status, error| {
                if let Err(error) = progress_handle.emit(
                    "enable-region-progress",
                    EnableRegionProgressEvent {
                        job_id: job_id_for_progress.clone(),
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                ) {
                    warn!("Failed to emit enable-region-progress: {}", error);
                }
            },
        )
        .await;

        match result {
            Ok(()) => {
                if let Some(provider_store) =
                    ProviderStore::open(&app_handle)
                {
                    provider_store.mark_region_enabled(&provider, &region);
                } else {
                    debug!(
                        "Provider store unavailable when marking region {} enabled for {}",
                        region, provider
                    );
                }
                if let Err(error) = app_handle.emit(
                    "enable-region-complete",
                    EnableRegionCompleteEvent {
                        job_id,
                        region,
                        provider: cloud_provider.get_provider_name(),
                    },
                ) {
                    warn!("Failed to emit enable-region-complete: {}", error);
                }
            }
            Err(error) => {
                if let Err(error) = app_handle.emit(
                    "enable-region-failed",
                    json!({ "jobId": &job_id, "error": error.to_string() }),
                ) {
                    warn!("Failed to emit enable-region-failed: {}", error);
                }
            }
        }
    });

    Ok(job)
}

#[tauri::command]
pub async fn get_regions(provider: String) -> Result<Vec<Region>> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name).await?;
    commands::setup::get_regions(&*cloud_provider).await
}

async fn fetch_vpn_status() -> Result<VpnStatus> {
    commands::status::fetch_vpn_status(&UnixDaemonClient).await
}

#[tauri::command]
pub async fn connect(
    instance_id: String,
    region: String,
    provider: String,
    public_ip_v4: Option<String>,
    public_ip_v6: Option<String>,
    app_handle: AppHandle,
) -> Result<String> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let cloud_provider = create_cloud_provider(provider_name).await?;
    let daemon_client = UnixDaemonClient;

    let kill_switch_enabled = SettingsStore::open(&app_handle)
        .map(|store| store.load_kill_switch_settings().kill_switch_enabled)
        .unwrap_or(false);
    if let Err(error) = daemon_client
        .send_command(DaemonCommand::SetKillSwitch { enabled: kill_switch_enabled })
        .await
    {
        warn!("Failed to sync kill switch state before connect: {}", error);
    }

    commands::connect::connect(
        &*cloud_provider,
        &daemon_client,
        region.as_str(),
        &instance_id,
        public_ip_v4,
        public_ip_v6,
    )
    .await?;

    let vpn_status = fetch_vpn_status().await?;

    if let Some(ref connected_instance) = vpn_status.instance {
        let emit_handle = app_handle.clone();
        let tray_handle = app_handle.clone();
        let settings_handle = app_handle.clone();
        let last_connected = connected_instance.clone();
        if let Err(error) = metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            vpn_status.connected_at,
            move |mut status| {
                if !status.connected {
                    let kill_switch_active = SettingsStore::open(&settings_handle)
                        .map(|store| store.load_kill_switch_settings().kill_switch_enabled)
                        .unwrap_or(false);
                    status = VpnStatus {
                        connected: true,
                        instance: Some(last_connected.clone()),
                        metrics: None,
                        connected_at: None,
                        connection_error: Some(if kill_switch_active {
                            "VPN tunnel dropped. Kill switch is blocking all traffic.".to_string()
                        } else {
                            "VPN connection lost. Please disconnect and reconnect.".to_string()
                        }),
                    };
                }
                tray::update_tray(&tray_handle, &status);
                let _ = emit_handle.emit("vpn-status", &status);
            },
        )
        .await
        {
            error!("Failed to start metrics stream: {}", error);
        }
    }

    tray::update_tray(&app_handle, &vpn_status);
    if let Err(error) = app_handle.emit("vpn-status", &vpn_status) {
        warn!("Failed to emit vpn-status: {}", error);
    }

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id
    ))
}

#[tauri::command]
pub async fn disconnect(app_handle: AppHandle) -> Result<String> {
    metrics_stream::stop().await?;

    let daemon_client = UnixDaemonClient;
    if daemon_client.is_daemon_running().await {
        commands::disconnect::disconnect(&daemon_client).await?;
    } else {
        if let Err(error) = byocvpn_daemon::firewall::remove() {
            warn!("Failed to remove firewall rules after daemon death: {}", error);
        }
    }

    let disconnected_status = VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
        connection_error: None,
    };
    tray::update_tray(&app_handle, &disconnected_status);
    if let Err(error) = app_handle.emit("vpn-status", &disconnected_status) {
        warn!("Failed to emit vpn-status: {}", error);
    }

    Ok("Disconnected successfully.".to_string())
}

#[tauri::command]
pub async fn get_vpn_status() -> Result<VpnStatus> {
    fetch_vpn_status().await
}

#[tauri::command]
pub async fn subscribe_to_vpn_status(app_handle: AppHandle) -> Result<()> {
    let status = fetch_vpn_status().await?;
    let connected_instance = status.instance.ok_or_else(|| -> Error {
        ConfigurationError::InvalidValue {
            field: "vpn_status".to_string(),
            reason: "not connected to VPN".to_string(),
        }
        .into()
    })?;

    let instance_id = connected_instance.instance_id.clone();
    let emit_handle = app_handle.clone();
    let tray_handle = app_handle.clone();
    let settings_handle = app_handle.clone();
    let last_connected = connected_instance.clone();
    let ledger_handle = app_handle;

    commands::subscribe::start_metrics_subscription(
        byocvpn_daemon::constants::metrics_socket_path(),
        connected_instance,
        status.connected_at,
        move |mut vpn_status| {
            if !vpn_status.connected {
                let kill_switch_active = SettingsStore::open(&settings_handle)
                    .map(|store| store.load_kill_switch_settings().kill_switch_enabled)
                    .unwrap_or(false);
                vpn_status = VpnStatus {
                    connected: true,
                    instance: Some(last_connected.clone()),
                    metrics: None,
                    connected_at: None,
                    connection_error: Some(if kill_switch_active {
                        "VPN tunnel dropped. Kill switch is blocking all traffic.".to_string()
                    } else {
                        "VPN connection lost. Please disconnect and reconnect.".to_string()
                    }),
                };
            }
            tray::update_tray(&tray_handle, &vpn_status);
            let _ = emit_handle.emit("vpn-status", &vpn_status);
        },
        move |bytes_sent, bytes_received| {
            if let Some(ledger) = LedgerStore::open(&ledger_handle) {
                ledger.update_metrics(&instance_id, bytes_sent, bytes_received);
            }
        },
    )
    .await
}

#[tauri::command]
pub async fn get_instance_pricing(provider: String, instance_type: String) -> Result<PricingInfo> {
    let provider_name = CloudProviderName::from_str(&provider)?;
    let pricing = match provider_name {
        CloudProviderName::Aws => aws_pricing::get_pricing(&instance_type),
        CloudProviderName::Azure => azure_pricing::get_pricing(&instance_type),
        CloudProviderName::Gcp => gcp_pricing::get_pricing(&instance_type),
        CloudProviderName::Oracle => oracle_pricing::get_pricing(&instance_type),
    };
    pricing.ok_or_else(|| {
        ConfigurationError::MissingField {
            field: format!("pricing/{}/{}", provider_name, instance_type),
        }
        .into()
    })
}

#[tauri::command]
pub async fn save_file(path: String, content: String) -> Result<()> {
    debug!("Writing file: {}", path);
    tokio::fs::write(&path, content)
        .await
        .map_err(|error| -> Error {
            ConfigurationError::InvalidFile {
                reason: error.to_string(),
            }
            .into()
        })
}

#[tauri::command]
pub async fn get_ledger(app_handle: AppHandle) -> Result<Vec<Value>> {
    let ledger = LedgerStore::open(&app_handle).ok_or_else(|| -> Error {
        ConfigurationError::InvalidFile {
            reason: "failed to open ledger store".to_string(),
        }
        .into()
    })?;
    Ok(ledger.all_entries())
}

