use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use byocvpn_core::{commands, ledger::LedgerEntry};
use chrono::Utc;
use humantime::format_duration;
use log::{debug, info, warn};
use tauri::{AppHandle, async_runtime};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;
use tokio::time::interval;

use crate::commands::{create_cloud_provider, fetch_vpn_status};
use crate::events::InstanceAutoTerminatedEvent;
use crate::ledger_store::LedgerStore;
use crate::settings_store::SettingsStore;

const CHECK_INTERVAL_SECONDS: u64 = 60;
const MIN_AUTO_TERMINATE_MINUTES: u64 = 5;

static LAST_NOTIFIED_AT: Mutex<Option<HashMap<String, SystemTime>>> = Mutex::new(None);

enum ConnectionState {
    Unknown,
    Disconnected,
    ConnectedTo(String),
}

pub fn start_server_monitor(app_handle: AppHandle) {
    async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECONDS));
        loop {
            ticker.tick().await;
            tick(&app_handle).await;
        }
    });
}

async fn tick(app_handle: &AppHandle) {
    let settings_store = match SettingsStore::open(app_handle) {
        Some(store) => store,
        None => return,
    };
    let notification_settings = settings_store.load_notification_settings();

    let running_entries = match LedgerStore::open(app_handle) {
        Some(ledger) => ledger.running_entries(),
        None => return,
    };
    if running_entries.is_empty() {
        return;
    }

    if notification_settings.notification_enabled {
        notify_long_running(
            app_handle,
            &running_entries,
            notification_settings.notification_threshold_minutes,
        );
    }

    run_auto_terminate_check(app_handle).await;
}

pub(crate) async fn run_auto_terminate_check(app_handle: &AppHandle) {
    let settings_store = match SettingsStore::open(app_handle) {
        Some(store) => store,
        None => return,
    };
    let ledger = match LedgerStore::open(app_handle) {
        Some(ledger) => ledger,
        None => return,
    };
    let running_entries = ledger.running_entries();
    if running_entries.is_empty() {
        return;
    }

    let connected_instance_id = match fetch_connection_state().await {
        ConnectionState::ConnectedTo(instance_id) => Some(instance_id),
        ConnectionState::Disconnected => None,
        ConnectionState::Unknown => return,
    };
    let running_entries =
        refresh_idle_state(&ledger, running_entries, connected_instance_id.as_deref());

    let auto_terminate_settings = settings_store.load_auto_terminate_settings();
    if !auto_terminate_settings.auto_terminate_enabled
        || auto_terminate_settings.auto_terminate_threshold_minutes == 0
    {
        return;
    }

    let notification_settings = settings_store.load_notification_settings();
    auto_terminate_idle(
        app_handle,
        &running_entries,
        connected_instance_id.as_deref(),
        auto_terminate_settings
            .auto_terminate_threshold_minutes
            .max(MIN_AUTO_TERMINATE_MINUTES),
        notification_settings.notification_enabled,
    )
    .await;
}

async fn fetch_connection_state() -> ConnectionState {
    let vpn_status = match fetch_vpn_status().await {
        Ok(status) => status,
        Err(error) => {
            warn!(
                "[auto-terminate] skipping this round — could not read VPN status, refusing to terminate a possibly-connected server: {}",
                error
            );
            return ConnectionState::Unknown;
        }
    };

    debug!(
        "[auto-terminate] raw daemon VpnStatus: connected={}, instance={:?}, connected_at={:?}",
        vpn_status.connected, vpn_status.instance, vpn_status.connected_at
    );

    if !vpn_status.connected {
        return ConnectionState::Disconnected;
    }
    match vpn_status.instance {
        Some(connected_instance) => ConnectionState::ConnectedTo(connected_instance.instance_id),
        None => {
            warn!(
                "[auto-terminate] skipping this round — VPN reports connected but no instance is identified"
            );
            ConnectionState::Unknown
        }
    }
}

fn refresh_idle_state(
    ledger: &LedgerStore,
    entries: Vec<LedgerEntry>,
    connected_instance_id: Option<&str>,
) -> Vec<LedgerEntry> {
    let now = Utc::now();
    entries
        .into_iter()
        .map(|mut entry| {
            let is_connected = connected_instance_id == Some(entry.instance_id.as_str());
            if is_connected && entry.idle_since.is_some() {
                ledger.mark_connected(&entry.instance_id);
                entry.mark_connected();
            } else if !is_connected && entry.idle_since.is_none() {
                ledger.mark_idle_since(&entry.instance_id, now);
                entry.mark_idle_since(now);
            }
            entry
        })
        .collect()
}

fn notify_long_running(app_handle: &AppHandle, entries: &[LedgerEntry], threshold_minutes: u64) {
    let threshold = Duration::from_secs(threshold_minutes * 60);
    let now = SystemTime::now();
    let now_utc = Utc::now();

    let mut last_notified = match LAST_NOTIFIED_AT.lock() {
        Ok(guard) => guard,
        Err(error) => {
            warn!("Failed to acquire uptime notifier lock: {error}");
            return;
        }
    };
    let map = last_notified.get_or_insert_with(HashMap::new);

    for entry in entries {
        let elapsed_secs = (now_utc - entry.launched_at).num_seconds().max(0) as u64;
        let elapsed = Duration::from_secs(elapsed_secs);
        if elapsed < threshold {
            continue;
        }

        let should_notify = match map.get(&entry.instance_id) {
            None => true,
            Some(last_time) => match now.duration_since(*last_time) {
                Ok(since_last) => since_last >= threshold,
                Err(_) => false,
            },
        };
        if !should_notify {
            continue;
        }

        map.insert(entry.instance_id.clone(), now);

        let provider = format!("{}", entry.provider).to_uppercase();
        let duration_text = format_duration(elapsed);
        let body = format!(
            "Your {provider} server in {} has been running for {duration_text}. Consider terminating it to avoid unnecessary costs.",
            entry.region,
        );

        if let Err(error) = app_handle
            .notification()
            .builder()
            .title("ByocVPN — Server Running")
            .body(&body)
            .show()
        {
            warn!("Failed to send uptime notification: {error}");
        }
    }
}

async fn auto_terminate_idle(
    app_handle: &AppHandle,
    entries: &[LedgerEntry],
    connected_instance_id: Option<&str>,
    threshold_minutes: u64,
    notifications_enabled: bool,
) {
    let threshold = chrono::Duration::minutes(threshold_minutes as i64);
    let now = Utc::now();

    info!(
        "[auto-terminate] round start: connected_instance_id={:?}, running_ledger_ids={:?}",
        connected_instance_id,
        entries
            .iter()
            .map(|entry| entry.instance_id.as_str())
            .collect::<Vec<_>>()
    );

    for entry in entries {
        if connected_instance_id == Some(entry.instance_id.as_str()) {
            info!(
                "[auto-terminate] skipping {} — VPN is currently connected to it",
                entry.instance_id
            );
            continue;
        }

        let idle_since = match entry.idle_since {
            Some(idle_since) => idle_since,
            None => continue,
        };
        let idle_duration = now - idle_since;
        debug!(
            "[auto-terminate] evaluating entry: ledger_id={:?}, idle_secs={}, threshold_secs={}, expired={}",
            entry.instance_id,
            idle_duration.num_seconds(),
            threshold.num_seconds(),
            idle_duration >= threshold
        );
        if idle_duration < threshold {
            continue;
        }

        info!(
            "[auto-terminate] {} ({}) idle for more than {}min; terminating",
            entry.instance_id, entry.provider, threshold_minutes
        );

        let cloud_provider = match create_cloud_provider(entry.provider.clone()).await {
            Ok(provider) => provider,
            Err(error) => {
                warn!(
                    "[auto-terminate] could not build provider for {}: {}",
                    entry.instance_id, error
                );
                continue;
            }
        };

        match commands::terminate::terminate_instance(
            &*cloud_provider,
            &entry.region,
            &entry.instance_id,
        )
        .await
        {
            Ok(_) => {
                if let Some(ledger) = LedgerStore::open(app_handle) {
                    ledger.mark_terminated(&entry.instance_id);
                }
                if notifications_enabled {
                    notify_terminated(app_handle, &entry.provider.to_string(), &entry.region);
                }
                if let Err(error) = (InstanceAutoTerminatedEvent {
                    instance_id: entry.instance_id.clone(),
                })
                .emit(app_handle)
                {
                    warn!("[auto-terminate] failed to emit event: {}", error);
                }
            }
            Err(error) => {
                warn!(
                    "[auto-terminate] failed to terminate {}: {}",
                    entry.instance_id, error
                );
            }
        }
    }
}

fn notify_terminated(app_handle: &AppHandle, provider: &str, region: &str) {
    let body = format!(
        "Your {} server in {} was automatically terminated after running idle without a VPN connection.",
        provider.to_uppercase(),
        region
    );
    if let Err(error) = app_handle
        .notification()
        .builder()
        .title("ByocVPN — Server Auto-Terminated")
        .body(&body)
        .show()
    {
        warn!("[auto-terminate] failed to send notification: {}", error);
    }
}
