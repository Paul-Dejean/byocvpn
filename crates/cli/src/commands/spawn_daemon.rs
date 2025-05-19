use byocvpn_core::{daemon::run_daemon, ipc::is_daemon_running};
pub async fn spawn_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if !is_daemon_running().await {
        println!("Starting embedded daemon...");
        run_daemon().await.expect("Daemon crashed");
    }
    Ok(())
}
