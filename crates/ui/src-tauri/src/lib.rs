mod commands;
mod ledger_store;
mod provider_store;
mod settings_store;
mod tray;
mod uptime_notifier;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Always)
        .init();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            tray::build_tray(app.handle())?;
            uptime_notifier::start_uptime_check_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
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
            settings_store::get_notification_settings,
            settings_store::save_notification_settings,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
