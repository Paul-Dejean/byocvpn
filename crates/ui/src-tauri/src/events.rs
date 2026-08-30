use byocvpn_core::tunnel::VpnStatus;
use serde::Serialize;
use tauri_specta::Event;

#[derive(Clone, Serialize, specta::Type, Event)]
#[tauri_specta(event_name = "vpn-status")]
pub struct VpnStatusEvent(pub VpnStatus);

#[derive(Clone, Serialize, specta::Type, Event)]
#[tauri_specta(event_name = "instance-auto-terminated")]
#[serde(rename_all = "camelCase")]
pub struct InstanceAutoTerminatedEvent {
    pub instance_id: String,
}
