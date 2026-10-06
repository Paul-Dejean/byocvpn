use byocvpn_core::tunnel::VpnStatus;
use log::warn;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::{events::VpnStatusEvent, tray};

#[cfg(desktop)]
mod desktop;
#[cfg(desktop)]
pub use desktop::{
    create_daemon_client, start_status_stream, start_status_subscription, stop_status_stream,
};

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::{
    create_daemon_client, start_status_stream, start_status_subscription, stop_status_stream,
};

pub fn publish_vpn_status(app_handle: &AppHandle, vpn_status: &VpnStatus) {
    tray::update_tray(app_handle, vpn_status);
    if let Err(error) = VpnStatusEvent(vpn_status.clone()).emit(app_handle) {
        warn!("Failed to emit vpn-status: {}", error);
    }
}
