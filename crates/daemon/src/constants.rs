use std::path::PathBuf;

pub fn socket_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".byocvpn/daemon.sock")
}

pub fn metrics_socket_path() -> PathBuf {
    socket_path()
        .parent()
        .expect("Socket path should have a parent directory")
        .join("metrics.sock")
}
