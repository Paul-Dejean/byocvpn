use crate::{
    cloud_provider::CloudProvider,
    credentials::get_configs_path,
    daemon_client::{DaemonClient, DaemonCommand},
    error::Result,
};

pub async fn connect(
    provider: &dyn CloudProvider,
    daemon_client: &dyn DaemonClient,
    instance_id: String,
) -> Result<()> {
    println!(
        "is daemon running: {}",
        daemon_client.is_daemon_running().await
    );

    let directory = get_configs_path().await?;
    let file_name = provider.get_config_file_name(&instance_id)?;
    let file_path = directory.join(file_name);

    // ✅ Now send the Connect command
    println!("Sending connect command to daemon...");
    let response = daemon_client
        .send_command(DaemonCommand::Connect {
            config_path: file_path.to_string_lossy().to_string(),
        })
        .await?;

    println!("Daemon response: {}", response);

    Ok(())
}
