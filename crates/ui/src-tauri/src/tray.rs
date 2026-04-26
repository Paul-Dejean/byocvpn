use std::{
    sync::{Mutex, atomic::{AtomicBool, Ordering}},
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};
use humantime::format_duration;

use byocvpn_core::tunnel::VpnStatus;
use log::warn;
use tauri::{
    AppHandle, Manager,
    menu::{MenuBuilder, MenuEvent, MenuItem, MenuItemBuilder},
    tray::TrayIconBuilder,
};

static LAST_CONNECTED_STATE: AtomicBool = AtomicBool::new(false);
static STATUS_MENU_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);

pub fn build_tray(app: &AppHandle) -> tauri::Result<tauri::tray::TrayIcon> {
    let (menu, status_item) = build_tray_menu(app, &disconnected_vpn_status())?;

    if let Ok(mut guard) = STATUS_MENU_ITEM.lock() {
        *guard = Some(status_item);
    }

    TrayIconBuilder::with_id("main")
        .icon(tauri::include_image!("icons/tray-disconnected.png"))
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("ByocVPN — Disconnected")
        .on_menu_event(move |app: &AppHandle, event: MenuEvent| match event.id().as_ref() {
            "open" => show_main_window(app),
            "disconnect" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = crate::commands::disconnect(app).await {
                        warn!("Tray disconnect failed: {error}");
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)
}

pub fn update_tray(app: &AppHandle, vpn_status: &VpnStatus) {
    let tray = match app.tray_by_id("main") {
        Some(tray) => tray,
        None => return,
    };

    let was_connected = LAST_CONNECTED_STATE.swap(vpn_status.connected, Ordering::Relaxed);

    let _ = tray.set_tooltip(Some(&build_tooltip(vpn_status)));

    if vpn_status.connected != was_connected {
        if vpn_status.connected {
            let _ = tray.set_icon(Some(tauri::include_image!("icons/tray-connected.png")));
        } else {
            let _ = tray.set_icon(Some(tauri::include_image!("icons/tray-disconnected.png")));
        }

        if let Ok((menu, status_item)) = build_tray_menu(app, vpn_status) {
            if let Ok(mut guard) = STATUS_MENU_ITEM.lock() {
                *guard = Some(status_item);
            }
            let _ = tray.set_menu(Some(menu));
        }
    } else if vpn_status.connected {
        if let Some(elapsed) = vpn_status.connected_at.and_then(|ts| get_elapsed_duration(ts).ok()) {
            let label = format!("Connected · {}", format_duration(elapsed));
            if let Ok(guard) = STATUS_MENU_ITEM.lock() {
                if let Some(ref item) = *guard {
                    let _ = item.set_text(&label);
                }
            }
        }
    }
}

fn build_tray_menu(
    app: &AppHandle,
    vpn_status: &VpnStatus,
) -> tauri::Result<(tauri::menu::Menu<tauri::Wry>, MenuItem<tauri::Wry>)> {
    let status_label = if vpn_status.connected {
        vpn_status
            .connected_at
            .and_then(|timestamp| get_elapsed_duration(timestamp).ok())
            .map(|elapsed| format!("Connected · {}", format_duration(elapsed)))
            .unwrap_or_else(|| "Connected".to_string())
    } else {
        "ByocVPN — Disconnected".to_string()
    };

    let status_item = MenuItemBuilder::new(&status_label)
        .id("status")
        .enabled(false)
        .build(app)?;

    let mut builder = MenuBuilder::new(app).item(&status_item);

    if vpn_status.connected {
        if let Some(ref instance) = vpn_status.instance {
            let provider_label = format!("{}", instance.provider).to_uppercase();
            let ip = instance.public_ip_v4.as_deref().unwrap_or("—");
            let info_text = format!("{} · {} · {}", provider_label, instance.region, ip);
            let info_item = MenuItemBuilder::new(info_text)
                .id("info")
                .enabled(false)
                .build(app)?;
            builder = builder.item(&info_item);
        }

        builder = builder.separator();

        let open_item = MenuItemBuilder::new("Open ByocVPN").id("open").build(app)?;
        let disconnect_item = MenuItemBuilder::new("Disconnect").id("disconnect").build(app)?;
        builder = builder.item(&open_item).item(&disconnect_item);
    } else {
        builder = builder.separator();

        let open_item = MenuItemBuilder::new("Open ByocVPN").id("open").build(app)?;
        builder = builder.item(&open_item);
    }

    builder = builder.separator();

    let quit_item = MenuItemBuilder::new("Quit").id("quit").build(app)?;
    let menu = builder.item(&quit_item).build()?;

    Ok((menu, status_item))
}

fn build_tooltip(vpn_status: &VpnStatus) -> String {
    if vpn_status.connected {
        vpn_status
            .connected_at
            .and_then(|timestamp| get_elapsed_duration(timestamp).ok())
            .map(|elapsed| format!("ByocVPN — Connected · {}", format_duration(elapsed)))
            .unwrap_or_else(|| "ByocVPN — Connected".to_string())
    } else {
        "ByocVPN — Disconnected".to_string()
    }
}

pub fn get_elapsed_duration(unix_secs: u64) -> Result<Duration, SystemTimeError> {
    SystemTime::now().duration_since(UNIX_EPOCH + Duration::from_secs(unix_secs))
}

fn disconnected_vpn_status() -> VpnStatus {
    VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
