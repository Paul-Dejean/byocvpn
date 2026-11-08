mod socket_paths;
pub use socket_paths::{metrics_socket_path, socket_path};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{IpcSocket, IpcStream};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{IpcSocket, IpcStream};
