#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{IpcSocket, IpcStream};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{IpcSocket, IpcStream};
