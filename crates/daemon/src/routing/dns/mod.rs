#[cfg(target_os = "linux")]
mod dns_linux;
#[cfg(target_os = "macos")]
mod dns_macos;
#[cfg(windows)]
mod dns_windows;

#[cfg(target_os = "linux")]
pub use dns_linux::DnsOverrideGuard;
#[cfg(target_os = "macos")]
pub use dns_macos::DnsOverrideGuard;
#[cfg(windows)]
pub use dns_windows::DnsOverrideGuard;
