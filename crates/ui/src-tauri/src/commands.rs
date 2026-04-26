use std::{collections::HashSet, str::FromStr};


use byocvpn_aws::{AwsCredentials, AwsProvider, pricing as aws_pricing};
use byocvpn_azure::{AzureProvider, credentials::AzureCredentials, pricing as azure_pricing};
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, EnableRegionCompleteEvent, EnableRegionJob,
        EnableRegionProgressEvent, InstanceInfo, PricingInfo, ProvisionAccountCompleteEvent,
        ProvisionAccountJob, ProvisionAccountProgressEvent, SpawnCompleteEvent, SpawnJob,
        SpawnProgressEvent,
    },
    commands,
    commands::setup::Region,
    credentials::CredentialStore,
    crypto::generate_keypair,
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
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::ledger_store::LedgerStore;
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
pub async fn get_credentials(provider: String) -> Result<Value> {
    let store = match CredentialStore::load().await {
        Ok(store) => store,
        Err(_) => return Ok(Value::Null),
    };
    fn serialize_or_null<T: serde::Serialize>(result: Result<T>) -> Value {
        result
            .ok()
            .and_then(|value| serde_json::to_value(value).ok())
            .unwrap_or(Value::Null)
    }
    let provider_name = CloudProviderName::from_str(&provider)?;
    Ok(match provider_name {
        CloudProviderName::Aws => serialize_or_null(AwsCredentials::from_store(&store)),
        CloudProviderName::Oracle => serialize_or_null(OracleCredentials::from_store(&store)),
        CloudProviderName::Gcp => serialize_or_null(GcpCredentials::from_store(&store)),
        CloudProviderName::Azure => serialize_or_null(AzureCredentials::from_store(&store)),
    })
}

#[tauri::command]
pub async fn save_credentials(provider: String, creds: Value) -> Result<()> {
    fn deserialize<T: serde::de::DeserializeOwned>(value: Value) -> Result<T> {
        serde_json::from_value(value).map_err(|error| {
            ConfigurationError::MissingField {
                field: error.to_string(),
            }
            .into()
        })
    }

    let mut store = CredentialStore::load().await?;
    let provider_name = CloudProviderName::from_str(&provider)?;

    match provider_name {
        CloudProviderName::Aws => deserialize::<AwsCredentials>(creds)?.write_to_store(&mut store),
        CloudProviderName::Oracle => {
            deserialize::<OracleCredentials>(creds)?.write_to_store(&mut store)
        }
        CloudProviderName::Gcp => deserialize::<GcpCredentials>(creds)?.write_to_store(&mut store),
        CloudProviderName::Azure => {
            deserialize::<AzureCredentials>(creds)?.write_to_store(&mut store)
        }
    }

    info!("Credentials saved for provider: {}", provider_name);
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
    if let Some(provider_store) = crate::provider_store::ProviderStore::open(&app_handle) {
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

    tauri::async_runtime::spawn(async move {
        let job_id_for_progress = job_id.clone();
        let progress_handle = app_handle.clone();
        let result = commands::spawn::run_spawn_steps(
            &*cloud_provider,
            &steps,
            &region,
            &client_private_key,
            &server_private_key,
            &client_public_key,
            &server_public_key,
            move |step_id, status, error| {
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
        )
        .await
        .and_then(|instance| {
            let entry = LedgerEntry {
                instance_id: instance.id.clone(),
                provider: provider_name.clone(),
                region: region.clone(),
                instance_type: instance.instance_type.clone(),
                launched_at: instance.launched_at.unwrap_or_else(Utc::now),
                terminated_at: None,
                bytes_sent: 0,
                bytes_received: 0,
            };
            Ok((instance, entry))
        });

        match result {
            Ok((instance, entry)) => {
                if let Some(ledger) = LedgerStore::open(&app_handle) {
                    ledger.set_entry(&entry);
                }
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
                    crate::provider_store::ProviderStore::open(&app_handle)
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
                    crate::provider_store::ProviderStore::open(&app_handle)
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
        if let Err(error) = metrics_stream::start(
            byocvpn_daemon::constants::metrics_socket_path(),
            connected_instance.clone(),
            vpn_status.connected_at,
            move |status| {
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
    commands::disconnect::disconnect(&daemon_client).await?;

    let disconnected_status = VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
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
    let ledger_handle = app_handle;

    commands::subscribe::start_metrics_subscription(
        byocvpn_daemon::constants::metrics_socket_path(),
        connected_instance,
        status.connected_at,
        move |vpn_status| {
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

