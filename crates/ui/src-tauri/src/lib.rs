mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_credentials,
            commands::save_credentials,
            commands::verify_permissions,
            commands::spawn_instance,
            commands::terminate_instance,
            commands::list_instances,
            commands::has_profile,
            commands::get_regions,
            commands::connect,
            commands::disconnect,
            commands::get_vpn_status,
            commands::subscribe_to_vpn_status,
            commands::get_instance_pricing,
            commands::get_ledger,
            commands::is_daemon_installed,
            commands::install_daemon,
        ]);

    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();
        builder = builder.plugin(devtools);
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
