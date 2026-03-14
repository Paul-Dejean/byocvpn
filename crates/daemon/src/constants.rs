use std::path::PathBuf;

fn socket_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("/var/run/byocvpn/dev")
    } else {
        PathBuf::from("/var/run/byocvpn/release")
    }
}

pub fn socket_path() -> PathBuf {
    socket_dir().join("daemon.sock")
}

pub fn metrics_socket_path() -> PathBuf {
    socket_dir().join("metrics.sock")
}
