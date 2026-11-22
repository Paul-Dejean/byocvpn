#[cfg(target_os = "android")]
use std::{env, path::Path, sync::Arc};

#[cfg(target_os = "android")]
use android_logger::Config as AndroidLoggerConfig;
#[cfg(desktop)]
use byocvpn_app::open_app_context;
#[cfg(target_os = "android")]
use byocvpn_app::open_app_context_with_daemon_client;
#[cfg(mobile)]
use byocvpn_core::config::set_data_directory;
#[cfg(desktop)]
use byocvpn_core::logging::{LoggingFormat, initialize_logging};
#[cfg(target_os = "android")]
use log::LevelFilter;
#[cfg(all(desktop, any(debug_assertions, test)))]
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{Commands, Events, collect_commands, collect_events};
use tracing::error;
#[cfg(desktop)]
use tracing::warn;

use crate::events::{InstanceAutoTerminatedEvent, VpnStatusEvent};

#[cfg(all(desktop, any(debug_assertions, test)))]
const TYPESCRIPT_BINDINGS_PATH: &str = "../src/bindings.ts";

#[cfg(target_os = "android")]
const CERTIFICATE_DIRECTORY_VARIABLE: &str = "SSL_CERT_DIR";

#[cfg(target_os = "android")]
const ANDROID_CERTIFICATE_DIRECTORIES: [&str; 2] = [
    "/apex/com.android.conscrypt/cacerts",
    "/system/etc/security/cacerts",
];

mod commands;
mod events;
mod job_observers;
mod server_monitor;
mod store_migration;
#[cfg(desktop)]
mod tray;
#[cfg(mobile)]
#[path = "tray_mobile.rs"]
mod tray;
mod vpn_backend;

fn build_specta_commands() -> Commands<tauri::Wry> {
    collect_commands![
        commands::get_credentials,
        commands::save_credentials,
        commands::delete_credentials,
        commands::verify_permissions,
        commands::spawn_instance,
        commands::terminate_instance,
        commands::list_instances,
        commands::has_profile,
        commands::provision_account,
        commands::enable_region,
        commands::get_regions,
        commands::connect,
        commands::disconnect,
        commands::get_vpn_status,
        commands::subscribe_to_vpn_status,
        commands::get_instance_pricing,
        commands::get_ledger,
        commands::save_file,
        commands::list_active_spawn_jobs,
        commands::dismiss_spawn_job,
        commands::get_enabled_regions,
        commands::get_notification_settings,
        commands::save_notification_settings,
        commands::get_auto_terminate_settings,
        commands::save_auto_terminate_settings,
        commands::get_vpn_settings,
        commands::save_vpn_settings,
        commands::get_ui_preferences,
        commands::save_ui_preferences,
    ]
}

fn build_specta_events() -> Events {
    collect_events![VpnStatusEvent, InstanceAutoTerminatedEvent]
}

fn build_specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(build_specta_commands())
        .events(build_specta_events())
}

#[cfg(all(desktop, any(debug_assertions, test)))]
fn export_typescript_bindings(
    specta_builder: &tauri_specta::Builder<tauri::Wry>,
) -> Result<(), specta_typescript::Error> {
    specta_builder.export(Typescript::default(), TYPESCRIPT_BINDINGS_PATH)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    configure_certificate_directory();
    initialize_logger();

    let specta_builder = build_specta_builder();

    #[cfg(all(debug_assertions, desktop))]
    if let Err(error) = export_typescript_bindings(&specta_builder) {
        warn!(%error, "failed to export TypeScript bindings");
    }

    let invoke_handler = specta_builder.invoke_handler();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "android")]
    let builder = builder.plugin(tauri_plugin_android_vpn::init());

    let builder = builder
        .setup(move |app| {
            #[cfg(mobile)]
            set_data_directory(app.path().app_data_dir()?);
            #[cfg(desktop)]
            let context = open_app_context()?;
            #[cfg(target_os = "android")]
            let context = open_app_context_with_daemon_client(Arc::new(
                vpn_backend::create_daemon_client(app.handle()),
            ))?;
            store_migration::import_legacy_stores(app.handle(), &context);
            app.manage(context);
            specta_builder.mount_events(app);
            #[cfg(desktop)]
            tray::build_tray(app.handle())?;
            server_monitor::start_server_monitor(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            let _ = (window, event);
            #[cfg(desktop)]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window.hide() {
                    warn!(%error, "failed to hide the main window");
                }
            }
        })
        .invoke_handler(invoke_handler);

    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(error) => {
            error!(%error, "error while building tauri application");
            return;
        }
    };

    app.run(|app_handle, event| {
        let _ = (&app_handle, &event);
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            tray::show_main_window(app_handle);
        }
    });
}

#[cfg(desktop)]
fn initialize_logger() {
    initialize_logging("info", LoggingFormat::DETAILED);
}

#[cfg(target_os = "android")]
fn initialize_logger() {
    android_logger::init_once(
        AndroidLoggerConfig::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("ByocVPN"),
    );
}

#[cfg(target_os = "android")]
fn configure_certificate_directory() {
    if env::var_os(CERTIFICATE_DIRECTORY_VARIABLE).is_some() {
        return;
    }
    let certificate_directory = ANDROID_CERTIFICATE_DIRECTORIES
        .iter()
        .find(|directory| Path::new(directory).is_dir());
    if let Some(certificate_directory) = certificate_directory {
        unsafe {
            env::set_var(CERTIFICATE_DIRECTORY_VARIABLE, certificate_directory);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() -> Result<(), specta_typescript::Error> {
        let specta_builder = build_specta_builder();
        export_typescript_bindings(&specta_builder)
    }
}
