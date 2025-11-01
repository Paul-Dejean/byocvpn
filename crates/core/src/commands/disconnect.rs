use crate::{
    ipc::{is_daemon_running, send_command},
    types::DaemonCommand,
};

pub async fn disconnect() -> Result<(), Box<dyn std::error::Error>> {
    println!("is daemon running: {}", is_daemon_running().await);
    if !is_daemon_running().await {
        println!("Starting embedded daemon...");
    }
    send_command(DaemonCommand::Disconnect).await?;
    Ok(())
}
