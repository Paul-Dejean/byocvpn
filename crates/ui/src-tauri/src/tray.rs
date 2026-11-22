use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use byocvpn_core::{duration_format::format_duration_compact, tunnel::VpnStatus};
use tauri::{
    AppHandle, Manager, Result as TauriResult, Wry,
    async_runtime::spawn,
    include_image,
    menu::{Menu, MenuBuilder, MenuEvent, MenuItem, MenuItemBuilder},
    tray::{TrayIcon, TrayIconBuilder},
};
use tracing::warn;

use crate::commands::disconnect;

static LAST_CONNECTED_STATE: AtomicBool = AtomicBool::new(false);
static STATUS_MENU_ITEM: Mutex<Option<MenuItem<Wry>>> = Mutex::new(None);

pub fn build_tray(app: &AppHandle) -> TauriResult<TrayIcon> {
    let (menu, status_item) = build_tray_menu(app, &disconnected_vpn_status())?;

    if let Ok(mut guard) = STATUS_MENU_ITEM.lock() {
        *guard = Some(status_item);
    }

    TrayIconBuilder::with_id("main")
        .icon(include_image!("icons/tray-disconnected.png"))
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("ByocVPN — Disconnected")
        .on_menu_event(
            move |menu_app: &AppHandle, event: MenuEvent| match event.id().as_ref() {
                "open" => show_main_window(menu_app),
                "disconnect" => {
                    let owned_app = menu_app.clone();
                    spawn(async move {
                        if let Err(error) = disconnect(owned_app).await {
                            warn!(%error, "tray disconnect failed");
                        }
                    });
                }
                "quit" => menu_app.exit(0),
                _ => {}
            },
        )
        .build(app)
}

pub fn update_tray(app: &AppHandle, vpn_status: &VpnStatus) {
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };

    let was_connected = LAST_CONNECTED_STATE.swap(vpn_status.connected, Ordering::Relaxed);

    if let Err(error) = tray.set_tooltip(Some(&build_tooltip(vpn_status))) {
        warn!(%error, "failed to update the tray tooltip");
    }

    if vpn_status.connected != was_connected {
        let icon = if vpn_status.connected {
            include_image!("icons/tray-connected.png")
        } else {
            include_image!("icons/tray-disconnected.png")
        };
        if let Err(error) = tray.set_icon(Some(icon)) {
            warn!(%error, "failed to update the tray icon");
        }

        if let Ok((menu, status_item)) = build_tray_menu(app, vpn_status) {
            if let Ok(mut guard) = STATUS_MENU_ITEM.lock() {
                *guard = Some(status_item);
            }
            if let Err(error) = tray.set_menu(Some(menu)) {
                warn!(%error, "failed to update the tray menu");
            }
        }
    } else if vpn_status.connected
        && let Some(elapsed) = vpn_status
            .connected_at
            .and_then(|timestamp| get_elapsed_duration(timestamp).ok())
    {
        let label = format!("Connected · {}", format_duration_compact(elapsed));
        if let Ok(guard) = STATUS_MENU_ITEM.lock()
            && let Some(item) = guard.as_ref()
            && let Err(error) = item.set_text(&label)
        {
            warn!(%error, "failed to update the tray status label");
        }
    }
}

fn build_tray_menu(
    app: &AppHandle,
    vpn_status: &VpnStatus,
) -> TauriResult<(Menu<Wry>, MenuItem<Wry>)> {
    let status_label = if vpn_status.connected {
        vpn_status
            .connected_at
            .and_then(|timestamp| get_elapsed_duration(timestamp).ok())
            .map_or_else(
                || "Connected".to_string(),
                |elapsed| format!("Connected · {}", format_duration_compact(elapsed)),
            )
    } else {
        "ByocVPN — Disconnected".to_string()
    };

    let status_item = MenuItemBuilder::new(&status_label)
        .id("status")
        .enabled(false)
        .build(app)?;

    let mut builder = MenuBuilder::new(app).item(&status_item);

    if vpn_status.connected {
        if let Some(instance) = vpn_status.instance.as_ref() {
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
        let disconnect_item = MenuItemBuilder::new("Disconnect")
            .id("disconnect")
            .build(app)?;
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
            .map_or_else(
                || "ByocVPN — Connected".to_string(),
                |elapsed| format!("ByocVPN — Connected · {}", format_duration_compact(elapsed)),
            )
    } else {
        "ByocVPN — Disconnected".to_string()
    }
}

pub fn get_elapsed_duration(unix_secs: u64) -> Result<Duration, SystemTimeError> {
    SystemTime::now().duration_since(UNIX_EPOCH + Duration::from_secs(unix_secs))
}

const fn disconnected_vpn_status() -> VpnStatus {
    VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
        connection_error: None,
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.show() {
            warn!(%error, "failed to show the main window");
        }
        if let Err(error) = window.set_focus() {
            warn!(%error, "failed to focus the main window");
        }
    }
}
