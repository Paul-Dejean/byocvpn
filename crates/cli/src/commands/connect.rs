use byocvpn_core::{
    ipc::{is_daemon_running, send_command},
    types::DaemonCommand,
};
use tokio::process::Command;

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    println!("is daemon running: {}", is_daemon_running().await);
    // if !is_daemon_running().await {
    //     Command::new(std::env::current_exe()?)
    //         .arg("spawn-daemon")
    //         .spawn()?;
    // }
    if is_daemon_running().await {
        send_command(DaemonCommand::Disconnect).await?;
        println!("Daemon disconnected");
    }

    Command::new(std::env::current_exe()?)
        .arg("spawn-daemon")
        .spawn()?;

    // ✅ Now send the Connect command
    println!("Sending connect command to daemon...");
    let response = send_command(DaemonCommand::Connect {
        config_path: "wg0.conf".to_string(), // or pass it from CLI argument
    })
    .await?;

    println!("Daemon response: {}", response);

    Ok(())
}
