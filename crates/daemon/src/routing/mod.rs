#[cfg(target_os = "linux")]
pub mod dns_linux;
#[cfg(target_os = "macos")]
pub mod dns_macos;
#[cfg(windows)]
pub mod dns_windows;
pub mod interface;
pub mod routes;
