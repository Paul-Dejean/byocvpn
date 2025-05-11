use byocvpn_core::daemon::run_daemon;
use byocvpn_core::ipc::{is_daemon_running, send_command};
use byocvpn_core::types::DaemonCommand;

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    println!("is daemon running: {}", is_daemon_running());
    if !is_daemon_running() {
        println!("Starting embedded daemon...");
        tokio::spawn(async {
            run_daemon().await.expect("Daemon crashed");
        });
    }

    // ✅ Now send the Connect command
    println!("Sending connect command to daemon...");
    let response = send_command(DaemonCommand::Connect {
        config_path: "wg0.conf".to_string(), // or pass it from CLI argument
    })
    .await?;

    println!("Daemon response: {}", response);

    Ok(())
}
