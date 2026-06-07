use std::sync::Arc;

use byocvpn_core::daemon_client::{DaemonClient, DaemonCommand};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use log::warn;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

const NOTIFICATION_ENABLED_KEY: &str = "notificationEnabled";
const NOTIFICATION_THRESHOLD_MINUTES_KEY: &str = "notificationThresholdMinutes";
const DEFAULT_THRESHOLD_MINUTES: u64 = 60;
const KILL_SWITCH_ENABLED_KEY: &str = "killSwitchEnabled";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub notification_enabled: bool,
    pub notification_threshold_minutes: u64,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            notification_enabled: false,
            notification_threshold_minutes: DEFAULT_THRESHOLD_MINUTES,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KillSwitchSettings {
    pub kill_switch_enabled: bool,
}

impl Default for KillSwitchSettings {
    fn default() -> Self {
        Self {
            kill_switch_enabled: false,
        }
    }
}

pub struct SettingsStore(Arc<Store<Wry>>);

impl SettingsStore {
    pub fn open(app_handle: &AppHandle) -> Option<Self> {
        app_handle.store("settings.json").ok().map(SettingsStore)
    }

    pub fn load_notification_settings(&self) -> NotificationSettings {
        let notification_enabled = self
            .0
            .get(NOTIFICATION_ENABLED_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(false);

        let notification_threshold_minutes = self
            .0
            .get(NOTIFICATION_THRESHOLD_MINUTES_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(DEFAULT_THRESHOLD_MINUTES);

        NotificationSettings {
            notification_enabled,
            notification_threshold_minutes,
        }
    }

    pub fn load_kill_switch_settings(&self) -> KillSwitchSettings {
        let kill_switch_enabled = self
            .0
            .get(KILL_SWITCH_ENABLED_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(false);
        KillSwitchSettings { kill_switch_enabled }
    }

    pub fn save_kill_switch_settings(&self, settings: &KillSwitchSettings) {
        self.0.set(
            KILL_SWITCH_ENABLED_KEY,
            serde_json::Value::Bool(settings.kill_switch_enabled),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save kill switch settings: {}", error);
        }
    }

    pub fn save_notification_settings(&self, settings: &NotificationSettings) {
        self.0.set(
            NOTIFICATION_ENABLED_KEY,
            serde_json::Value::Bool(settings.notification_enabled),
        );
        self.0.set(
            NOTIFICATION_THRESHOLD_MINUTES_KEY,
            serde_json::Value::Number(settings.notification_threshold_minutes.into()),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save notification settings: {}", error);
        }
    }
}

#[tauri::command]
pub fn get_notification_settings(app_handle: AppHandle) -> NotificationSettings {
    SettingsStore::open(&app_handle)
        .map(|store| store.load_notification_settings())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_notification_settings(
    app_handle: AppHandle,
    settings: NotificationSettings,
) -> Result<(), String> {
    match SettingsStore::open(&app_handle) {
        Some(store) => {
            store.save_notification_settings(&settings);
            Ok(())
        }
        None => Err("Failed to open settings store".to_string()),
    }
}

#[tauri::command]
pub fn get_kill_switch_settings(app_handle: AppHandle) -> KillSwitchSettings {
    SettingsStore::open(&app_handle)
        .map(|store| store.load_kill_switch_settings())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn save_kill_switch_settings(
    app_handle: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let settings = KillSwitchSettings { kill_switch_enabled: enabled };
    match SettingsStore::open(&app_handle) {
        Some(store) => {
            store.save_kill_switch_settings(&settings);
        }
        None => return Err("Failed to open settings store".to_string()),
    }

    let client = UnixDaemonClient;
    if client.is_daemon_running().await {
        if let Err(error) = client
            .send_command(DaemonCommand::SetKillSwitch { enabled })
            .await
        {
            warn!("Failed to send SetKillSwitch to daemon: {}", error);
        }
    }

    Ok(())
}
