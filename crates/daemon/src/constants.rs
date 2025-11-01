use std::path::PathBuf;

pub fn socket_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".byocvpn/daemon.sock")
}
