use std::path::PathBuf;

pub const TUNNEL_MTU: u16 = 1280;

pub fn get_interface_name() -> &'static str {
    #[cfg(target_os = "macos")]
    return "utun4";
    #[cfg(windows)]
    return "byocvpn";
    #[cfg(not(any(target_os = "macos", windows)))]
    return "tun0";
}

#[cfg(unix)]
fn socket_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("/var/run/byocvpn/dev")
    } else {
        PathBuf::from("/var/run/byocvpn/release")
    }
}

#[cfg(unix)]
pub fn socket_path() -> PathBuf {
    socket_dir().join("daemon.sock")
}

#[cfg(unix)]
pub fn metrics_socket_path() -> PathBuf {
    socket_dir().join("metrics.sock")
}

#[cfg(windows)]
fn pipe_prefix() -> &'static str {
    if cfg!(debug_assertions) {
        r"\\.\pipe\byocvpn\dev"
    } else {
        r"\\.\pipe\byocvpn\release"
    }
}

#[cfg(windows)]
pub fn socket_path() -> PathBuf {
    PathBuf::from(format!(r"{}\daemon", pipe_prefix()))
}

#[cfg(windows)]
pub fn metrics_socket_path() -> PathBuf {
    PathBuf::from(format!(r"{}\metrics", pipe_prefix()))
}
