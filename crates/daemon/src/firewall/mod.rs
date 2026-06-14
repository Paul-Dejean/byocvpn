use byocvpn_core::error::Result;
use std::sync::Mutex;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub struct KillSwitchState {
    pub server_ip: Option<String>,
    pub tun_name: Option<String>,
}

pub static KILL_SWITCH: Mutex<KillSwitchState> = Mutex::new(KillSwitchState {
    server_ip: None,
    tun_name: None,
});

pub fn apply(server_ip: &str, tun_name: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    macos::apply(server_ip, tun_name)?;
    #[cfg(target_os = "linux")]
    linux::apply(server_ip, tun_name)?;
    #[cfg(target_os = "windows")]
    windows::apply(server_ip, tun_name)?;

    if let Ok(mut state) = KILL_SWITCH.lock() {
        state.server_ip = Some(server_ip.to_string());
        state.tun_name = Some(tun_name.to_string());
    }
    Ok(())
}

pub fn remove() -> Result<()> {
    #[cfg(target_os = "macos")]
    macos::remove()?;
    #[cfg(target_os = "linux")]
    linux::remove()?;
    #[cfg(target_os = "windows")]
    windows::remove()?;

    if let Ok(mut state) = KILL_SWITCH.lock() {
        state.server_ip = None;
        state.tun_name = None;
    }
    Ok(())
}
