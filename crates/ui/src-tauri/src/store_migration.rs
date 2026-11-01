use std::{collections::HashMap, fs, path::Path, str::FromStr};

use byocvpn_core::{
    app_context::AppContext,
    cloud_provider::CloudProviderName,
    ledger::LedgerEntry,
    settings::{AutoTerminateSettings, NotificationSettings, UiPreferences, VpnSettings},
};
use log::{info, warn};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Manager};

const LEGACY_LEDGER_FILE: &str = "byocvpn-ledger.json";
const LEGACY_SETTINGS_FILE: &str = "settings.json";
const LEGACY_PROVIDERS_FILE: &str = "providers.json";
const LEGACY_UPDATES_FILE: &str = "updates.json";
const LEGACY_LEDGER_KEY_PREFIX: &str = "ledger/";
const LEGACY_PROVISIONED_KEY_PREFIX: &str = "provisioned/";
const LEGACY_ENABLED_REGION_KEY_PREFIX: &str = "enabled_regions/";
const LEGACY_FAVORITE_REGIONS_KEY_PREFIX: &str = "favorite_regions/";

type LegacyStore = HashMap<String, Value>;

pub fn import_legacy_stores(app_handle: &AppHandle, context: &AppContext) {
    let legacy_directory = match app_handle.path().app_data_dir() {
        Ok(directory) => directory,
        Err(error) => {
            warn!("Skipping legacy store import, app data directory unavailable: {error}");
            return;
        }
    };

    if !context.ledger.is_persisted()
        && let Some(legacy_ledger) = read_legacy_store(&legacy_directory.join(LEGACY_LEDGER_FILE))
    {
        import_ledger(&legacy_ledger, context);
    }

    if !context.settings.is_persisted() {
        if let Some(legacy_settings) =
            read_legacy_store(&legacy_directory.join(LEGACY_SETTINGS_FILE))
        {
            import_settings(&legacy_settings, context);
        }
        let legacy_providers = read_legacy_store(&legacy_directory.join(LEGACY_PROVIDERS_FILE));
        let legacy_updates = read_legacy_store(&legacy_directory.join(LEGACY_UPDATES_FILE));
        if legacy_providers.is_some() || legacy_updates.is_some() {
            import_ui_preferences(legacy_providers.as_ref(), legacy_updates.as_ref(), context);
        }
    }

    if !context.provider_flags.is_persisted()
        && let Some(legacy_providers) =
            read_legacy_store(&legacy_directory.join(LEGACY_PROVIDERS_FILE))
    {
        import_provider_flags(&legacy_providers, context);
    }
}

fn import_ledger(legacy_ledger: &LegacyStore, context: &AppContext) {
    let mut imported_count = 0;
    for (key, value) in legacy_ledger {
        if !key.starts_with(LEGACY_LEDGER_KEY_PREFIX) {
            continue;
        }
        match serde_json::from_value::<LedgerEntry>(value.clone()) {
            Ok(entry) => {
                context.ledger.insert_entry(entry);
                imported_count += 1;
            }
            Err(error) => warn!("Skipping unreadable legacy ledger entry {key}: {error}"),
        }
    }
    info!("Imported {imported_count} ledger entries from the legacy store");
}

fn import_settings(legacy_settings: &LegacyStore, context: &AppContext) {
    let vpn_defaults = VpnSettings::default();
    context.settings.save_vpn_settings(VpnSettings {
        session_killswitch: read_legacy_value(legacy_settings, "sessionKillswitch")
            .unwrap_or(vpn_defaults.session_killswitch),
    });

    let notification_defaults = NotificationSettings::default();
    context
        .settings
        .save_notification_settings(NotificationSettings {
            notification_enabled: read_legacy_value(legacy_settings, "notificationEnabled")
                .unwrap_or(notification_defaults.notification_enabled),
            notification_threshold_minutes: read_legacy_value(
                legacy_settings,
                "notificationThresholdMinutes",
            )
            .unwrap_or(notification_defaults.notification_threshold_minutes),
            notification_unit: read_legacy_value(legacy_settings, "notificationUnit")
                .unwrap_or(notification_defaults.notification_unit),
        });

    let auto_terminate_defaults = AutoTerminateSettings::default();
    context
        .settings
        .save_auto_terminate_settings(AutoTerminateSettings {
            auto_terminate_enabled: read_legacy_value(legacy_settings, "autoTerminateEnabled")
                .unwrap_or(auto_terminate_defaults.auto_terminate_enabled),
            auto_terminate_threshold_minutes: read_legacy_value(
                legacy_settings,
                "autoTerminateThresholdMinutes",
            )
            .unwrap_or(auto_terminate_defaults.auto_terminate_threshold_minutes),
            auto_terminate_unit: read_legacy_value(legacy_settings, "autoTerminateUnit")
                .unwrap_or(auto_terminate_defaults.auto_terminate_unit),
        });
    info!("Imported settings from the legacy store");
}

fn import_ui_preferences(
    legacy_providers: Option<&LegacyStore>,
    legacy_updates: Option<&LegacyStore>,
    context: &AppContext,
) {
    let mut preferences = UiPreferences::default();

    if let Some(legacy_providers) = legacy_providers {
        for (key, value) in legacy_providers {
            if let Some(provider_text) = key.strip_prefix(LEGACY_FAVORITE_REGIONS_KEY_PREFIX)
                && let Ok(provider) = CloudProviderName::from_str(provider_text)
                && let Ok(regions) = serde_json::from_value::<Vec<String>>(value.clone())
            {
                preferences.favorite_regions.insert(provider, regions);
            }
        }
        preferences.last_deploy_provider =
            read_legacy_value(legacy_providers, "deploy/last_provider");
        preferences.disconnect_reminder_dismissal_count =
            read_legacy_value(legacy_providers, "reminders/disconnect_dismissal_count")
                .unwrap_or(preferences.disconnect_reminder_dismissal_count);
    }

    if let Some(legacy_updates) = legacy_updates {
        preferences.update_check_enabled =
            read_legacy_value(legacy_updates, "updates/check_enabled")
                .unwrap_or(preferences.update_check_enabled);
        preferences.dismissed_update_version =
            read_legacy_value(legacy_updates, "updates/dismissed_version");
    }

    context.settings.save_ui_preferences(preferences);
    info!("Imported UI preferences from the legacy stores");
}

fn import_provider_flags(legacy_providers: &LegacyStore, context: &AppContext) {
    for (key, value) in legacy_providers {
        if value != &Value::Bool(true) {
            continue;
        }
        if let Some(provider_text) = key.strip_prefix(LEGACY_PROVISIONED_KEY_PREFIX)
            && let Ok(provider) = CloudProviderName::from_str(provider_text)
        {
            context.provider_flags.mark_provisioned(&provider);
        } else if let Some(provider_and_region) = key.strip_prefix(LEGACY_ENABLED_REGION_KEY_PREFIX)
            && let Some((provider_text, region)) = provider_and_region.split_once('/')
            && let Ok(provider) = CloudProviderName::from_str(provider_text)
        {
            context
                .provider_flags
                .mark_region_enabled(&provider, region);
        }
    }
    info!("Imported provider flags from the legacy store");
}

fn read_legacy_store(path: &Path) -> Option<LegacyStore> {
    let bytes = fs::read(path).ok()?;
    match serde_json::from_slice(&bytes) {
        Ok(store) => Some(store),
        Err(error) => {
            warn!(
                "Ignoring unreadable legacy store {}: {error}",
                path.display()
            );
            None
        }
    }
}

fn read_legacy_value<Value: DeserializeOwned>(store: &LegacyStore, key: &str) -> Option<Value> {
    store
        .get(key)
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}
