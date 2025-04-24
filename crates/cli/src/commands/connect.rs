use std::process::Command;

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    Command::new("wg-quick")
        .args(["up", "./wg0.conf"])
        .status()
        .expect("Failed to bring up WireGuard interface");
    Ok(())
}
