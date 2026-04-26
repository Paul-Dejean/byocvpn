use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use chrono::Utc;
use humantime::format_duration;
use log::warn;
use tauri::{async_runtime, AppHandle};
use tauri_plugin_notification::NotificationExt;
use tokio::time::interval;

use crate::ledger_store::LedgerStore;
use crate::settings_store::SettingsStore;

static LAST_NOTIFIED_AT: Mutex<Option<HashMap<String, SystemTime>>> = Mutex::new(None);

pub fn start_uptime_check_loop(app_handle: AppHandle) {
    async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            check_and_notify(&app_handle);
        }
    });
}

fn check_and_notify(app_handle: &AppHandle) {
    let settings = match SettingsStore::open(app_handle) {
        Some(store) => store.load_notification_settings(),
        None => return,
    };

    if !settings.notification_enabled {
        return;
    }

    let running_entries = match LedgerStore::open(app_handle) {
        Some(store) => store.running_entries(),
        None => return,
    };

    let threshold = Duration::from_secs(settings.notification_threshold_minutes * 60);
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

    for entry in running_entries {
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
