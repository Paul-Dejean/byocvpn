use byocvpn_core::tunnel::VpnStatus;
use tauri::AppHandle;

pub fn update_tray(_app_handle: &AppHandle, _vpn_status: &VpnStatus) {}
