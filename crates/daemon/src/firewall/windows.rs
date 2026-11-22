use byocvpn_core::error::Result;

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature matches the other platforms' apply"
)]
pub const fn apply(_server_ip: &str, _tun_name: &str) -> Result<()> {
    Ok(())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature matches the other platforms' remove"
)]
pub const fn remove() -> Result<()> {
    Ok(())
}
