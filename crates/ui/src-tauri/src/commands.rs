use byocvpn_app::ProviderCredentials;
use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::{CloudProviderName, InstanceInfo, PermissionStatus, PricingInfo},
    commands,
    commands::setup::Region,
    error::{ConfigurationError, CredentialsError, Error, Result},
    instances,
    job_registry::SpawnJobState,
    jobs::{
        enable_region::run_enable_region_job, provision_account::run_provision_account_job,
        spawn::run_spawn_job,
    },
    ledger::LedgerEntry,
    session::{self, ConnectTarget},
    settings::{AutoTerminateSettings, NotificationSettings, UiPreferences, VpnSettings},
    tunnel::VpnStatus,
};
use log::{debug, error};
use tauri::{AppHandle, Manager, ipc::Channel};

use crate::job_observers::{
    EnableRegionEvent, EnableRegionEventForwarder, ProvisionAccountEvent,
    ProvisionAccountEventForwarder, SpawnEventForwarder, SpawnInstanceEvent,
};
use crate::vpn_backend::{self, publish_vpn_status};

#[tauri::command]
#[specta::specta]
pub async fn get_credentials(provider: CloudProviderName) -> Result<Option<ProviderCredentials>> {
    match ProviderCredentials::load(provider).await {
        Ok(credentials) => Ok(Some(credentials)),
        Err(Error::Credentials(CredentialsError::MissingField { .. })) => Ok(None),
        Err(error) => Err(error),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn save_credentials(credentials: ProviderCredentials) -> Result<()> {
    credentials.save().await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_credentials(provider: CloudProviderName, app_handle: AppHandle) -> Result<()> {
    ProviderCredentials::delete(provider.clone()).await?;
    app_handle
        .state::<AppContext>()
        .provider_flags
        .clear_provisioned(&provider);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn verify_permissions(
    provider: CloudProviderName,
    app_handle: AppHandle,
) -> Result<Vec<PermissionStatus>> {
    let cloud_provider = app_handle
        .state::<AppContext>()
        .providers
        .create(provider)
        .await?;
    commands::verify_permissions::verify_permissions(&*cloud_provider).await
}

#[tauri::command]
#[specta::specta]
pub async fn spawn_instance(
    region: String,
    provider: CloudProviderName,
    on_event: Channel<SpawnInstanceEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = app_handle
        .state::<AppContext>()
        .providers
        .create(provider)
        .await?;
    let observer = SpawnEventForwarder::new(app_handle.clone(), on_event);

    tauri::async_runtime::spawn(async move {
        let context = app_handle.state::<AppContext>();
        if let Err(error) = run_spawn_job(&*cloud_provider, &context, &region, &observer).await {
            debug!("Spawn job in {region} ended with error: {error}");
        }
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn list_active_spawn_jobs(app_handle: AppHandle) -> Result<Vec<SpawnJobState>> {
    Ok(app_handle.state::<AppContext>().spawn_jobs.list())
}

#[tauri::command]
#[specta::specta]
pub async fn dismiss_spawn_job(job_id: String, app_handle: AppHandle) -> Result<()> {
    app_handle
        .state::<AppContext>()
        .spawn_jobs
        .deregister(&job_id);
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
    let context = app_handle.state::<AppContext>();
    instances::terminate_instance(&context, provider, &region, &instance_id).await?;
    Ok(format!("Instance {instance_id} terminated successfully."))
}

#[tauri::command]
#[specta::specta]
pub async fn list_instances(
    region: Option<String>,
    app_handle: AppHandle,
) -> Result<Vec<InstanceInfo>> {
    let context = app_handle.state::<AppContext>();
    let listing = instances::list_instances(&context, None, region.as_deref()).await?;
    for failure in &listing.failures {
        error!(
            "Failed to list {} instances: {}",
            failure.provider, failure.error
        );
    }
    Ok(listing.instances)
}

#[tauri::command]
#[specta::specta]
pub async fn has_profile(app_handle: AppHandle) -> Result<bool> {
    let configured_providers = app_handle
        .state::<AppContext>()
        .providers
        .list_configured()
        .await?;
    Ok(!configured_providers.is_empty())
}

#[tauri::command]
#[specta::specta]
pub async fn provision_account(
    provider: CloudProviderName,
    on_event: Channel<ProvisionAccountEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = app_handle
        .state::<AppContext>()
        .providers
        .create(provider.clone())
        .await?;
    let observer = ProvisionAccountEventForwarder::new(on_event);

    tauri::async_runtime::spawn(async move {
        let context = app_handle.state::<AppContext>();
        if let Err(error) = run_provision_account_job(&*cloud_provider, &context, &observer).await {
            debug!("Provision account job for {provider} ended with error: {error}");
        }
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn enable_region(
    region: String,
    provider: CloudProviderName,
    on_event: Channel<EnableRegionEvent>,
    app_handle: AppHandle,
) -> Result<()> {
    let cloud_provider = app_handle
        .state::<AppContext>()
        .providers
        .create(provider.clone())
        .await?;
    let observer = EnableRegionEventForwarder::new(on_event);

    tauri::async_runtime::spawn(async move {
        let context = app_handle.state::<AppContext>();
        if let Err(error) =
            run_enable_region_job(&*cloud_provider, &context, &region, &observer).await
        {
            debug!("Enable region job for {provider} {region} ended with error: {error}");
        }
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_regions(
    provider: CloudProviderName,
    app_handle: AppHandle,
) -> Result<Vec<Region>> {
    let cloud_provider = app_handle
        .state::<AppContext>()
        .providers
        .create(provider)
        .await?;
    commands::setup::get_regions(&*cloud_provider).await
}

pub async fn fetch_vpn_status(app_handle: &AppHandle) -> Result<VpnStatus> {
    commands::status::fetch_vpn_status(&vpn_backend::create_daemon_client(app_handle)).await
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
    let daemon_client = vpn_backend::create_daemon_client(&app_handle);
    let context = app_handle.state::<AppContext>();
    session::connect(
        &context,
        &daemon_client,
        ConnectTarget {
            provider,
            region,
            instance_id: instance_id.clone(),
            public_ip_v4,
            public_ip_v6,
        },
    )
    .await?;

    let vpn_status = fetch_vpn_status(&app_handle).await?;

    if let Some(ref connected_instance) = vpn_status.instance
        && let Err(error) = vpn_backend::start_status_stream(
            &app_handle,
            connected_instance.clone(),
            vpn_status.connected_at,
        )
        .await
    {
        error!("Failed to start metrics stream: {error}");
    }

    publish_vpn_status(&app_handle, &vpn_status);

    Ok(format!("Connected to instance {instance_id} successfully."))
}

#[tauri::command]
#[specta::specta]
pub async fn disconnect(app_handle: AppHandle) -> Result<String> {
    vpn_backend::stop_status_stream().await?;

    let daemon_client = vpn_backend::create_daemon_client(&app_handle);
    session::disconnect(&daemon_client).await?;

    let disconnected_status = VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
        connection_error: None,
    };
    publish_vpn_status(&app_handle, &disconnected_status);

    crate::server_monitor::run_auto_terminate_check(&app_handle).await;

    Ok("Disconnected successfully.".to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_vpn_status(app_handle: AppHandle) -> Result<VpnStatus> {
    fetch_vpn_status(&app_handle).await
}

#[tauri::command]
#[specta::specta]
pub async fn subscribe_to_vpn_status(app_handle: AppHandle) -> Result<()> {
    let status = fetch_vpn_status(&app_handle).await?;
    let connected_instance = status.instance.ok_or_else(|| -> Error {
        ConfigurationError::InvalidValue {
            field: "vpn_status".to_string(),
            reason: "not connected to VPN".to_string(),
        }
        .into()
    })?;

    vpn_backend::start_status_subscription(&app_handle, connected_instance, status.connected_at)
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn get_instance_pricing(
    provider: CloudProviderName,
    instance_type: String,
    app_handle: AppHandle,
) -> Result<PricingInfo> {
    let context = app_handle.state::<AppContext>();
    instances::get_instance_pricing(&context, provider, &instance_type).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_file(path: String, content: String) -> Result<()> {
    debug!("Writing file: {path}");
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
    Ok(app_handle.state::<AppContext>().ledger.all_entries())
}

#[tauri::command]
#[specta::specta]
pub fn get_enabled_regions(provider: CloudProviderName, app_handle: AppHandle) -> Vec<String> {
    app_handle
        .state::<AppContext>()
        .provider_flags
        .list_enabled_regions(&provider)
}

#[tauri::command]
#[specta::specta]
pub fn get_notification_settings(app_handle: AppHandle) -> NotificationSettings {
    app_handle
        .state::<AppContext>()
        .settings
        .load_notification_settings()
}

#[tauri::command]
#[specta::specta]
pub fn save_notification_settings(app_handle: AppHandle, settings: NotificationSettings) {
    app_handle
        .state::<AppContext>()
        .settings
        .save_notification_settings(settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_auto_terminate_settings(app_handle: AppHandle) -> AutoTerminateSettings {
    app_handle
        .state::<AppContext>()
        .settings
        .load_auto_terminate_settings()
}

#[tauri::command]
#[specta::specta]
pub fn save_auto_terminate_settings(app_handle: AppHandle, settings: AutoTerminateSettings) {
    app_handle
        .state::<AppContext>()
        .settings
        .save_auto_terminate_settings(settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_vpn_settings(app_handle: AppHandle) -> VpnSettings {
    app_handle
        .state::<AppContext>()
        .settings
        .load_vpn_settings()
}

#[tauri::command]
#[specta::specta]
pub fn save_vpn_settings(app_handle: AppHandle, settings: VpnSettings) {
    app_handle
        .state::<AppContext>()
        .settings
        .save_vpn_settings(settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_ui_preferences(app_handle: AppHandle) -> UiPreferences {
    app_handle
        .state::<AppContext>()
        .settings
        .load_ui_preferences()
}

#[tauri::command]
#[specta::specta]
pub fn save_ui_preferences(app_handle: AppHandle, preferences: UiPreferences) {
    app_handle
        .state::<AppContext>()
        .settings
        .save_ui_preferences(preferences);
}
