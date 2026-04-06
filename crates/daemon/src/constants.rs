use std::path::PathBuf;

#[cfg(unix)]
fn socket_dir() -> PathBuf {
    if cfg!(feature = "external-daemon") {
        PathBuf::from("/var/run/byocvpn/external")
    } else if cfg!(debug_assertions) {
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
    if cfg!(feature = "external-daemon") {
        r"\\.\pipe\byocvpn\external"
    } else if cfg!(debug_assertions) {
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
