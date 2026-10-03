use specta_typescript::Typescript;
use tauri_specta::{Commands, Events, collect_commands, collect_events};

use crate::events::{InstanceAutoTerminatedEvent, VpnStatusEvent};

const TYPESCRIPT_BINDINGS_PATH: &str = "../src/bindings.ts";

mod commands;
mod events;
mod ledger_store;
mod provider_credentials;
mod provider_store;
mod server_monitor;
mod settings_store;
mod spawn_job_registry;
mod tray;

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
        settings_store::get_notification_settings,
        settings_store::save_notification_settings,
        settings_store::get_auto_terminate_settings,
        settings_store::save_auto_terminate_settings,
        settings_store::get_vpn_settings,
        settings_store::save_vpn_settings,
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

fn export_typescript_bindings(
    specta_builder: &tauri_specta::Builder<tauri::Wry>,
) -> Result<(), specta_typescript::Error> {
    specta_builder.export(Typescript::default(), TYPESCRIPT_BINDINGS_PATH)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Always)
        .init();

    let specta_builder = build_specta_builder();

    #[cfg(debug_assertions)]
    if let Err(error) = export_typescript_bindings(&specta_builder) {
        log::error!("Failed to export TypeScript bindings: {error}");
    }

    let invoke_handler = specta_builder.invoke_handler();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(spawn_job_registry::SpawnJobRegistry::new())
        .setup(move |app| {
            specta_builder.mount_events(app);
            tray::build_tray(app.handle())?;
            server_monitor::start_server_monitor(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(invoke_handler);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            tray::show_main_window(_app_handle);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        let specta_builder = build_specta_builder();
        export_typescript_bindings(&specta_builder)
            .unwrap_or_else(|error| panic!("failed to export TypeScript bindings: {error}"));
    }
}
