mod commands;
mod ledger_store;
mod provider_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
