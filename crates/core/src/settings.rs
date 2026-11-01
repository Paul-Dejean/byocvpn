use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{cloud_provider::CloudProviderName, error::Result, json_store::JsonFileStore};

const DEFAULT_NOTIFICATION_THRESHOLD_MINUTES: u64 = 60;
const DEFAULT_AUTO_TERMINATE_THRESHOLD_MINUTES: u64 = 720;
const DEFAULT_SESSION_KILLSWITCH: bool = true;
const DEFAULT_UPDATE_CHECK_ENABLED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum DurationUnit {
    Minutes,
    Hours,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub notification_enabled: bool,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub notification_threshold_minutes: u64,
    pub notification_unit: DurationUnit,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            notification_enabled: false,
            notification_threshold_minutes: DEFAULT_NOTIFICATION_THRESHOLD_MINUTES,
            notification_unit: DurationUnit::Minutes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct AutoTerminateSettings {
    pub auto_terminate_enabled: bool,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub auto_terminate_threshold_minutes: u64,
    pub auto_terminate_unit: DurationUnit,
}

impl Default for AutoTerminateSettings {
    fn default() -> Self {
        Self {
            auto_terminate_enabled: false,
            auto_terminate_threshold_minutes: DEFAULT_AUTO_TERMINATE_THRESHOLD_MINUTES,
            auto_terminate_unit: DurationUnit::Hours,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct UiPreferences {
    pub favorite_regions: BTreeMap<CloudProviderName, Vec<String>>,
    pub last_deploy_provider: Option<CloudProviderName>,
    pub disconnect_reminder_dismissal_count: u32,
    pub update_check_enabled: bool,
    pub dismissed_update_version: Option<String>,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            favorite_regions: BTreeMap::new(),
            last_deploy_provider: None,
            disconnect_reminder_dismissal_count: 0,
            update_check_enabled: DEFAULT_UPDATE_CHECK_ENABLED,
            dismissed_update_version: None,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct SettingsData {
    vpn: VpnSettings,
    notifications: NotificationSettings,
    auto_terminate: AutoTerminateSettings,
    ui_preferences: UiPreferences,
}

pub struct Settings {
    store: JsonFileStore<SettingsData>,
}

impl Settings {
    pub const FILE_NAME: &'static str = "app-settings.json";

    pub fn open() -> Result<Self> {
        Ok(Self {
            store: JsonFileStore::open(Self::FILE_NAME)?,
        })
    }

    pub fn is_persisted(&self) -> bool {
        self.store.is_persisted()
    }

    pub fn load_vpn_settings(&self) -> VpnSettings {
        self.store.read(|data| data.vpn.clone())
    }

    pub fn save_vpn_settings(&self, settings: VpnSettings) {
        self.store.modify(|data| data.vpn = settings);
    }

    pub fn load_notification_settings(&self) -> NotificationSettings {
        self.store.read(|data| data.notifications.clone())
    }

    pub fn save_notification_settings(&self, settings: NotificationSettings) {
        self.store.modify(|data| data.notifications = settings);
    }

    pub fn load_auto_terminate_settings(&self) -> AutoTerminateSettings {
        self.store.read(|data| data.auto_terminate.clone())
    }

    pub fn save_auto_terminate_settings(&self, settings: AutoTerminateSettings) {
        self.store.modify(|data| data.auto_terminate = settings);
    }

    pub fn load_ui_preferences(&self) -> UiPreferences {
        self.store.read(|data| data.ui_preferences.clone())
    }

    pub fn save_ui_preferences(&self, preferences: UiPreferences) {
        self.store.modify(|data| data.ui_preferences = preferences);
    }
}
