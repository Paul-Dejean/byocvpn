use std::collections::HashSet;

use byocvpn_aws::{AwsCredentials, AwsProvider, pricing as aws_pricing};
use byocvpn_azure::{AzureProvider, credentials::AzureCredentials, pricing as azure_pricing};
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, InstanceState, PermissionStatus,
        PricingInfo, SpawnJob, SpawnStep, SpawnStepStatus,
    },
    commands,
    commands::setup::Region,
    connectivity::{self, ProbeStatus},
    credentials::{CredentialStore, StoredCredentials},
    crypto::generate_keypair,
    daemon_client::DaemonClient,
    error::{ConfigurationError, Error, Result},
    ledger::LedgerEntry,
    metrics_stream,
    tunnel::VpnStatus,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use byocvpn_gcp::{GcpProvider, credentials::GcpCredentials, pricing as gcp_pricing};
use byocvpn_oracle::{credentials::OracleCredentials, pricing as oracle_pricing};
use chrono::Utc;
use log::*;
use serde::Serialize;
use tauri::{AppHandle, Manager, ipc::Channel};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;

use crate::events::VpnStatusEvent;
use crate::ledger_store::LedgerStore;
use crate::provider_credentials::ProviderCredentials;
use crate::provider_store::ProviderStore;
use crate::spawn_job_registry::{SpawnJobRegistry, SpawnJobState};
use crate::tray;

pub(crate) async fn create_cloud_provider(
    provider: CloudProviderName,
) -> Result<Box<dyn CloudProvider>> {
    debug!("Creating {} cloud provider", provider);
    let store = CredentialStore::load().await?;
    let provider: Box<dyn CloudProvider> = match provider {
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
#[specta::specta]
pub async fn get_credentials(provider: CloudProviderName) -> Result<Option<ProviderCredentials>> {
    let store = match CredentialStore::load().await {
        Ok(store) => store,
        Err(_) => return Ok(None),
    };
    Ok(ProviderCredentials::load(provider, &store).ok())
}

#[tauri::command]
#[specta::specta]
pub async fn save_credentials(credentials: ProviderCredentials) -> Result<()> {
    let mut store = CredentialStore::load().await?;
    credentials.write_to_store(&mut store);
    store.save()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_credentials(provider: CloudProviderName, app_handle: AppHandle) -> Result<()> {
    let mut store = CredentialStore::load().await?;
    let section = match provider {
        CloudProviderName::Aws => AwsCredentials::CREDENTIALS_SECTION,
        CloudProviderName::Oracle => OracleCredentials::CREDENTIALS_SECTION,
        CloudProviderName::Gcp => GcpCredentials::CREDENTIALS_SECTION,
        CloudProviderName::Azure => AzureCredentials::CREDENTIALS_SECTION,
    };
    store.delete_section(section);
    store.save()?;
    if let Some(provider_store) = ProviderStore::open(&app_handle) {
        provider_store.clear_provisioned(&provider.to_string());
    } else {
        debug!(
            "Provider store unavailable when deleting credentials for {}",
            provider
        );
    }
    Ok(())
}

async fn create_cloud_provider_from_credentials(
    credentials: ProviderCredentials,
) -> Result<Box<dyn CloudProvider>> {
    let provider: Box<dyn CloudProvider> = match credentials {
        ProviderCredentials::Aws(aws_credentials) => {
            Box::new(AwsProvider::new(aws_credentials.into()).await)
        }
        ProviderCredentials::Gcp(gcp_credentials) => {
            Box::new(GcpProvider::new(gcp_credentials.into())?)
        }
        ProviderCredentials::Oracle(oracle_credentials) => Box::new(
            byocvpn_oracle::OracleProvider::new(oracle_credentials.into()),
        ),
        ProviderCredentials::Azure(azure_credentials) => {
            Box::new(AzureProvider::new(azure_credentials.into())?)
        }
    };
    Ok(provider)
}

#[tauri::command]
#[specta::specta]
pub async fn verify_permissions(
    provider: CloudProviderName,
    credentials: Option<ProviderCredentials>,
) -> Result<Vec<PermissionStatus>> {
    let cloud_provider: Box<dyn CloudProvider> = match credentials {
        Some(credentials) => create_cloud_provider_from_credentials(credentials).await?,
        None => create_cloud_provider(provider).await?,
    };
    commands::verify_permissions::verify_permissions(&*cloud_provider).await
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(specta::Type)]
pub enum SpawnInstanceEvent {
    Started {
        job: SpawnJobState,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: SpawnStepStatus,
        error: Option<String>,
    },
    InstanceLaunched {
        instance: InstanceInfo,
    },
    Complete {
        instance: InstanceInfo,
    },
    Failed {
        error: String,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn spawn_instance(
    region: String,
    provider: CloudProviderName,
    on_event: Channel<SpawnInstanceEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = create_cloud_provider(provider.clone()).await?;

    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let job = SpawnJob {
        job_id: format!("{}-{}", provider, Utc::now().timestamp_millis()),
        steps: cloud_provider.get_spawn_steps(&region),
        region: region.clone(),
        provider: provider.clone(),
    };

    let job_id = job.job_id.clone();
    let steps = job.steps.clone();

    let job_state = app_handle.state::<SpawnJobRegistry>().register(job);
    send_spawn_instance_event(&on_event, SpawnInstanceEvent::Started { job: job_state });

    tauri::async_runtime::spawn(async move {
        let job_id_for_progress = job_id.clone();
        let job_id_for_launched = job_id.clone();
        let progress_handle = app_handle.clone();
        let launched_handle = app_handle.clone();
        let progress_channel = on_event.clone();
        let launched_channel = on_event.clone();
        let region_for_launched = region.clone();
        let provider_for_launched = provider.clone();

        let result = commands::spawn::run_spawn_steps(
            &*cloud_provider,
            &steps,
            &region,
            &job_id,
            &client_private_key,
            &server_private_key,
            &client_public_key,
            &server_public_key,
            move |step_id, status, error| {
                progress_handle
                    .state::<SpawnJobRegistry>()
                    .update_step_status(
                        &job_id_for_progress,
                        step_id,
                        status.clone(),
                        error.clone(),
                    );

                send_spawn_instance_event(
                    &progress_channel,
                    SpawnInstanceEvent::Progress {
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                );
            },
            move |instance| {
                launched_handle
                    .state::<SpawnJobRegistry>()
                    .set_instance_id(&job_id_for_launched, instance.id.clone());

                if let Some(ledger) = LedgerStore::open(&launched_handle) {
                    let entry = LedgerEntry {
                        instance_id: instance.id.clone(),
                        provider: provider_for_launched.clone(),
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
                send_spawn_instance_event(
                    &launched_channel,
                    SpawnInstanceEvent::InstanceLaunched {
                        instance: installing_instance,
                    },
                );
            },
        )
        .await;

        match result {
            Ok(mut instance) => {
                app_handle.state::<SpawnJobRegistry>().deregister(&job_id);
                if let Some(ledger) = LedgerStore::open(&app_handle) {
                    ledger.mark_setup_complete(&instance.id);
                }
                instance.state = InstanceState::Running;
                notify_server_deployed(&app_handle, &instance);
                send_spawn_instance_event(&on_event, SpawnInstanceEvent::Complete { instance });
            }
            Err(error) => {
                app_handle
                    .state::<SpawnJobRegistry>()
                    .mark_failed(&job_id, error.to_string());
                send_spawn_instance_event(
                    &on_event,
                    SpawnInstanceEvent::Failed {
                        error: error.to_string(),
                    },
                );
            }
        }
    });

    Ok(())
}

fn send_spawn_instance_event(channel: &Channel<SpawnInstanceEvent>, event: SpawnInstanceEvent) {
    if let Err(error) = channel.send(event) {
        warn!("Failed to send spawn-instance event: {}", error);
    }
}

fn notify_server_deployed(app_handle: &AppHandle, instance: &InstanceInfo) {
    if has_focused_window(app_handle) {
        return;
    }

    let body = format!(
        "Your {} server in {} is ready to connect.",
        instance.provider.to_string().to_uppercase(),
        instance.region
    );
    if let Err(error) = app_handle
        .notification()
        .builder()
        .title("ByocVPN — Server Ready")
        .body(&body)
        .show()
    {
        warn!("Failed to send server-deployed notification: {}", error);
    }
}

fn has_focused_window(app_handle: &AppHandle) -> bool {
    app_handle.webview_windows().values().any(|window| {
        window.is_visible().unwrap_or(false)
            && window.is_focused().unwrap_or(false)
            && !window.is_minimized().unwrap_or(false)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_active_spawn_jobs(app_handle: AppHandle) -> Result<Vec<SpawnJobState>> {
    Ok(app_handle.state::<SpawnJobRegistry>().list())
}

#[tauri::command]
#[specta::specta]
pub async fn dismiss_spawn_job(job_id: String, app_handle: AppHandle) -> Result<()> {
    app_handle.state::<SpawnJobRegistry>().deregister(&job_id);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn terminate_instance(
    instance_id: String,
    region: String,
    provider: CloudProviderName,
    app_handle: AppHandle,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(provider).await?;
    commands::terminate::terminate_instance(&*cloud_provider, &region, &instance_id).await?;

    if let Some(ledger) = LedgerStore::open(&app_handle) {
        ledger.mark_terminated(&instance_id);
    }

    Ok(format!("Instance {} terminated successfully.", instance_id))
}

#[tauri::command]
#[specta::specta]
pub async fn list_instances(
    region: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<InstanceInfo>> {
    let region_ref = region.as_deref();

    async fn list_provider_instances(
        provider: CloudProviderName,
        region: Option<&str>,
    ) -> (CloudProviderName, Option<Vec<InstanceInfo>>) {
        match create_cloud_provider(provider.clone()).await {
            Ok(cloud_provider) => {
                match commands::list::list_instances(&*cloud_provider, region).await {
                    Ok(instances) => (provider, Some(instances)),
                    Err(error) => {
                        error!("Failed to list {} instances: {}", provider, error);
                        (provider, None)
                    }
                }
            }
            Err(error) => {
                debug!("No credentials for {}, skipping: {}", provider, error);
                (provider, None)
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
    let mut queried_providers: Vec<CloudProviderName> = Vec::new();
    for (provider, result) in [r_aws, r_oracle, r_gcp, r_azure] {
        if let Some(instances) = result {
            queried_providers.push(provider);
            all_instances.extend(instances);
        }
    }

    if let Some(ledger) = LedgerStore::open(&app_handle) {
        let running_ids: HashSet<&str> = all_instances.iter().map(|i| i.id.as_str()).collect();
        ledger.reconcile_terminated(&running_ids, &queried_providers);

        let in_progress_ids = app_handle
            .state::<SpawnJobRegistry>()
            .instance_ids_in_progress();

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
#[specta::specta]
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

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(specta::Type)]
pub enum ProvisionAccountEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        job_id: String,
        steps: Vec<SpawnStep>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: SpawnStepStatus,
        error: Option<String>,
    },
    Complete {
        provider: CloudProviderName,
    },
    Failed {
        error: String,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn provision_account(
    provider: CloudProviderName,
    on_event: Channel<ProvisionAccountEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = create_cloud_provider(provider.clone()).await?;

    let job_id = format!("{}-{}", provider, Utc::now().timestamp_millis());
    let steps = cloud_provider.get_provision_account_steps();

    send_provision_account_event(
        &on_event,
        ProvisionAccountEvent::Started {
            job_id,
            steps: steps.clone(),
        },
    );

    tauri::async_runtime::spawn(async move {
        let progress_channel = on_event.clone();

        let result = commands::setup::run_provision_account_steps(
            &*cloud_provider,
            &steps,
            move |step_id, status, error| {
                send_provision_account_event(
                    &progress_channel,
                    ProvisionAccountEvent::Progress {
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                );
            },
        )
        .await;

        match result {
            Ok(()) => {
                if let Some(provider_store) = ProviderStore::open(&app_handle) {
                    provider_store.mark_provisioned(&provider.to_string());
                } else {
                    debug!(
                        "Provider store unavailable when marking {} provisioned",
                        provider
                    );
                }
                send_provision_account_event(
                    &on_event,
                    ProvisionAccountEvent::Complete {
                        provider: cloud_provider.get_provider_name(),
                    },
                );
            }
            Err(error) => {
                send_provision_account_event(
                    &on_event,
                    ProvisionAccountEvent::Failed {
                        error: error.to_string(),
                    },
                );
            }
        }
    });

    Ok(())
}

fn send_provision_account_event(
    channel: &Channel<ProvisionAccountEvent>,
    event: ProvisionAccountEvent,
) {
    if let Err(error) = channel.send(event) {
        warn!("Failed to send provision-account event: {}", error);
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(specta::Type)]
pub enum EnableRegionEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        job_id: String,
        steps: Vec<SpawnStep>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: SpawnStepStatus,
        error: Option<String>,
    },
    Complete {
        region: String,
    },
    Failed {
        error: String,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn enable_region(
    region: String,
    provider: CloudProviderName,
    on_event: Channel<EnableRegionEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = create_cloud_provider(provider.clone()).await?;

    let job_id = format!("{}-{}-{}", provider, region, Utc::now().timestamp_millis());
    let steps = cloud_provider.get_enable_region_steps(&region);

    send_enable_region_event(
        &on_event,
        EnableRegionEvent::Started {
            job_id,
            steps: steps.clone(),
        },
    );

    tauri::async_runtime::spawn(async move {
        let progress_channel = on_event.clone();

        let result = commands::setup::run_enable_region_steps(
            &*cloud_provider,
            &steps,
            &region,
            move |step_id, status, error| {
                send_enable_region_event(
                    &progress_channel,
                    EnableRegionEvent::Progress {
                        step_id: step_id.to_string(),
                        status,
                        error,
                    },
                );
            },
        )
        .await;

        match result {
            Ok(()) => {
                if let Some(provider_store) = ProviderStore::open(&app_handle) {
                    provider_store.mark_region_enabled(&provider.to_string(), &region);
                } else {
                    debug!(
                        "Provider store unavailable when marking region {} enabled for {}",
                        region, provider
                    );
                }
                send_enable_region_event(&on_event, EnableRegionEvent::Complete { region });
            }
            Err(error) => {
                send_enable_region_event(
                    &on_event,
                    EnableRegionEvent::Failed {
                        error: error.to_string(),
                    },
                );
            }
        }
    });

    Ok(())
}

fn send_enable_region_event(channel: &Channel<EnableRegionEvent>, event: EnableRegionEvent) {
    if let Err(error) = channel.send(event) {
        warn!("Failed to send enable-region event: {}", error);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_regions(provider: CloudProviderName) -> Result<Vec<Region>> {
    let cloud_provider = create_cloud_provider(provider).await?;
    commands::setup::get_regions(&*cloud_provider).await
}

pub(crate) async fn fetch_vpn_status() -> Result<VpnStatus> {
    commands::status::fetch_vpn_status(&UnixDaemonClient).await
}

#[tauri::command]
#[specta::specta]
pub async fn connect(
    instance_id: String,
    region: String,
    provider: CloudProviderName,
    public_ip_v4: Option<String>,
    public_ip_v6: Option<String>,
    app_handle: AppHandle,
) -> Result<String> {
    let cloud_provider = create_cloud_provider(provider).await?;
    let daemon_client = UnixDaemonClient;

    let kill_switch_enabled = crate::settings_store::SettingsStore::open(&app_handle)
        .map(|store| store.load_vpn_settings().session_killswitch)
        .unwrap_or(true);

    commands::connect::connect(
        &*cloud_provider,
        &daemon_client,
        region.as_str(),
        &instance_id,
        public_ip_v4,
        public_ip_v6,
        kill_switch_enabled,
    )
    .await?;

    let vpn_status = fetch_vpn_status().await?;

    if let Some(ref connected_instance) = vpn_status.instance {
        let emit_handle = app_handle.clone();
        let tray_handle = app_handle.clone();
        let last_connected = connected_instance.clone();
        if let Err(error) = metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            vpn_status.connected_at,
            move |mut status| {
                if !status.connected {
                    status = VpnStatus {
                        connected: true,
                        instance: Some(last_connected.clone()),
                        metrics: None,
                        connected_at: None,
                        connection_error: Some(
                            "VPN tunnel dropped. Kill switch is blocking all traffic.".to_string(),
                        ),
                    };
                }
                tray::update_tray(&tray_handle, &status);
                let _ = VpnStatusEvent(status.clone()).emit(&emit_handle);
            },
        )
        .await
        {
            error!("Failed to start metrics stream: {}", error);
        }
    }

    tray::update_tray(&app_handle, &vpn_status);
    if let Err(error) = VpnStatusEvent(vpn_status.clone()).emit(&app_handle) {
        warn!("Failed to emit vpn-status: {}", error);
    }

    Ok(format!(
        "Connected to instance {} successfully.",
        instance_id
    ))
}

#[tauri::command]
#[specta::specta]
pub async fn disconnect(app_handle: AppHandle) -> Result<String> {
    metrics_stream::stop().await?;

    let daemon_client = UnixDaemonClient;
    if daemon_client.is_daemon_running().await {
        commands::disconnect::disconnect(&daemon_client).await?;
    } else {
        return Err(byocvpn_core::error::Error::Daemon(
            byocvpn_core::error::DaemonError::NotRunning,
        ));
    }

    let disconnected_status = VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
        connection_error: None,
    };
    tray::update_tray(&app_handle, &disconnected_status);
    if let Err(error) = VpnStatusEvent(disconnected_status.clone()).emit(&app_handle) {
        warn!("Failed to emit vpn-status: {}", error);
    }

    crate::server_monitor::run_auto_terminate_check(&app_handle).await;

    Ok("Disconnected successfully.".to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_vpn_status() -> Result<VpnStatus> {
    fetch_vpn_status().await
}

#[tauri::command]
#[specta::specta]
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
    let last_connected = connected_instance.clone();
    let ledger_handle = app_handle;

    commands::subscribe::start_metrics_subscription(
        byocvpn_daemon::constants::metrics_socket_path(),
        connected_instance,
        status.connected_at,
        move |mut vpn_status| {
            if !vpn_status.connected {
                vpn_status = VpnStatus {
                    connected: true,
                    instance: Some(last_connected.clone()),
                    metrics: None,
                    connected_at: None,
                    connection_error: Some(
                        "VPN tunnel dropped. Kill switch is blocking all traffic.".to_string(),
                    ),
                };
            }
            tray::update_tray(&tray_handle, &vpn_status);
            let _ = VpnStatusEvent(vpn_status.clone()).emit(&emit_handle);
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
#[specta::specta]
pub async fn get_instance_pricing(
    provider: CloudProviderName,
    instance_type: String,
) -> Result<PricingInfo> {
    let pricing = match provider {
        CloudProviderName::Aws => aws_pricing::get_pricing(&instance_type),
        CloudProviderName::Azure => azure_pricing::get_pricing(&instance_type),
        CloudProviderName::Gcp => gcp_pricing::get_pricing(&instance_type),
        CloudProviderName::Oracle => oracle_pricing::get_pricing(&instance_type),
    };
    pricing.ok_or_else(|| {
        ConfigurationError::MissingField {
            field: format!("pricing/{}/{}", provider, instance_type),
        }
        .into()
    })
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
pub async fn get_ledger(app_handle: AppHandle) -> Result<Vec<LedgerEntry>> {
    let ledger = LedgerStore::open(&app_handle).ok_or_else(|| -> Error {
        ConfigurationError::InvalidFile {
            reason: "failed to open ledger store".to_string(),
        }
        .into()
    })?;
    Ok(ledger.all_entries())
}
