use std::sync::Arc;

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

const NOTIFICATION_ENABLED_KEY: &str = "notificationEnabled";
const NOTIFICATION_THRESHOLD_MINUTES_KEY: &str = "notificationThresholdMinutes";
const DEFAULT_THRESHOLD_MINUTES: u64 = 60;

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
