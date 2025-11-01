use tokio::process::Command;

use crate::{
    CloudProvider, get_configs_path,
    ipc::{is_daemon_running, send_command},
    types::DaemonCommand,
};

pub async fn connect(
    provider: &dyn CloudProvider,
    instance_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("is daemon running: {}", is_daemon_running().await);
    // if !is_daemon_running().await {
    //     Command::new(std::env::current_exe()?)
    //         .arg("spawn-daemon")
    //         .spawn()?;
    // }
    let directory = get_configs_path().await?;
    let file_name = provider.get_config_file_name(&instance_id)?;
    let file_path = directory.join(file_name);

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
        config_path: file_path.to_string_lossy().to_string(),
    })
    .await?;

    println!("Daemon response: {}", response);

    Ok(())
}
