use std::process::Command;

pub async fn disconnect() -> Result<(), Box<dyn std::error::Error>> {
    Command::new("wg-quick")
        .args(["down", "./wg0.conf"])
        .status()
        .expect("Failed to bring up WireGuard interface");
    Ok(())
}
