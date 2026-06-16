use std::sync::Arc;

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

const NOTIFICATION_ENABLED_KEY: &str = "notificationEnabled";
const NOTIFICATION_THRESHOLD_MINUTES_KEY: &str = "notificationThresholdMinutes";
const NOTIFICATION_UNIT_KEY: &str = "notificationUnit";
const DEFAULT_THRESHOLD_MINUTES: u64 = 60;
const DEFAULT_NOTIFICATION_UNIT: &str = "minutes";
const SESSION_KILLSWITCH_KEY: &str = "sessionKillswitch";
const DEFAULT_SESSION_KILLSWITCH: bool = true;
const AUTO_TERMINATE_ENABLED_KEY: &str = "autoTerminateEnabled";
const AUTO_TERMINATE_THRESHOLD_MINUTES_KEY: &str = "autoTerminateThresholdMinutes";
const AUTO_TERMINATE_UNIT_KEY: &str = "autoTerminateUnit";
const DEFAULT_AUTO_TERMINATE_THRESHOLD_MINUTES: u64 = 720;
const DEFAULT_AUTO_TERMINATE_UNIT: &str = "hours";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnSettings {
    pub session_killswitch: bool,
}

impl Default for VpnSettings {
    fn default() -> Self {
        Self {
            session_killswitch: DEFAULT_SESSION_KILLSWITCH,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub notification_enabled: bool,
    pub notification_threshold_minutes: u64,
    pub notification_unit: String,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            notification_enabled: false,
            notification_threshold_minutes: DEFAULT_THRESHOLD_MINUTES,
            notification_unit: DEFAULT_NOTIFICATION_UNIT.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTerminateSettings {
    pub auto_terminate_enabled: bool,
    pub auto_terminate_threshold_minutes: u64,
    pub auto_terminate_unit: String,
}

impl Default for AutoTerminateSettings {
    fn default() -> Self {
        Self {
            auto_terminate_enabled: false,
            auto_terminate_threshold_minutes: DEFAULT_AUTO_TERMINATE_THRESHOLD_MINUTES,
            auto_terminate_unit: DEFAULT_AUTO_TERMINATE_UNIT.to_string(),
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

        let notification_unit = self
            .0
            .get(NOTIFICATION_UNIT_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_else(|| DEFAULT_NOTIFICATION_UNIT.to_string());

        NotificationSettings {
            notification_enabled,
            notification_threshold_minutes,
            notification_unit,
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
        self.0.set(
            NOTIFICATION_UNIT_KEY,
            serde_json::Value::String(settings.notification_unit.clone()),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save notification settings: {}", error);
        }
    }

    pub fn load_auto_terminate_settings(&self) -> AutoTerminateSettings {
        let auto_terminate_enabled = self
            .0
            .get(AUTO_TERMINATE_ENABLED_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(false);

        let auto_terminate_threshold_minutes = self
            .0
            .get(AUTO_TERMINATE_THRESHOLD_MINUTES_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(DEFAULT_AUTO_TERMINATE_THRESHOLD_MINUTES);

        let auto_terminate_unit = self
            .0
            .get(AUTO_TERMINATE_UNIT_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_else(|| DEFAULT_AUTO_TERMINATE_UNIT.to_string());

        AutoTerminateSettings {
            auto_terminate_enabled,
            auto_terminate_threshold_minutes,
            auto_terminate_unit,
        }
    }

    pub fn save_auto_terminate_settings(&self, settings: &AutoTerminateSettings) {
        self.0.set(
            AUTO_TERMINATE_ENABLED_KEY,
            serde_json::Value::Bool(settings.auto_terminate_enabled),
        );
        self.0.set(
            AUTO_TERMINATE_THRESHOLD_MINUTES_KEY,
            serde_json::Value::Number(settings.auto_terminate_threshold_minutes.into()),
        );
        self.0.set(
            AUTO_TERMINATE_UNIT_KEY,
            serde_json::Value::String(settings.auto_terminate_unit.clone()),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save auto-terminate settings: {}", error);
        }
    }

    pub fn load_vpn_settings(&self) -> VpnSettings {
        let session_killswitch = self
            .0
            .get(SESSION_KILLSWITCH_KEY)
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or(DEFAULT_SESSION_KILLSWITCH);

        VpnSettings { session_killswitch }
    }

    pub fn save_vpn_settings(&self, settings: &VpnSettings) {
        self.0.set(
            SESSION_KILLSWITCH_KEY,
            serde_json::Value::Bool(settings.session_killswitch),
        );
        if let Err(error) = self.0.save() {
            warn!("Failed to save VPN settings: {}", error);
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
pub fn get_auto_terminate_settings(app_handle: AppHandle) -> AutoTerminateSettings {
    SettingsStore::open(&app_handle)
        .map(|store| store.load_auto_terminate_settings())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_auto_terminate_settings(
    app_handle: AppHandle,
    settings: AutoTerminateSettings,
) -> Result<(), String> {
    match SettingsStore::open(&app_handle) {
        Some(store) => {
            store.save_auto_terminate_settings(&settings);
            Ok(())
        }
        None => Err("Failed to open settings store".to_string()),
    }
}

#[tauri::command]
pub fn get_vpn_settings(app_handle: AppHandle) -> VpnSettings {
    SettingsStore::open(&app_handle)
        .map(|store| store.load_vpn_settings())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_vpn_settings(app_handle: AppHandle, settings: VpnSettings) -> Result<(), String> {
    match SettingsStore::open(&app_handle) {
        Some(store) => {
            store.save_vpn_settings(&settings);
            Ok(())
        }
        None => Err("Failed to open settings store".to_string()),
    }
}
