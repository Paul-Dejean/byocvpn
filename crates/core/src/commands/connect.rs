use crate::{
    cloud_provider::CloudProvider,
    config::get_wireguard_config_file_path,
    daemon_client::{DaemonClient, DaemonCommand},
    error::Result,
};
use log::*;
pub async fn connect(
    provider: &dyn CloudProvider,
    daemon_client: &dyn DaemonClient,
    region: &str,
    instance_id: &str,
) -> Result<()> {
    let provider_name = provider.get_provider_name();
    let wireguard_file_path =
        get_wireguard_config_file_path(&provider_name, region, instance_id).await?;

    info!("Sending connect command to daemon...");
    let response = daemon_client
        .send_command(DaemonCommand::Connect {
            config_path: wireguard_file_path.to_string_lossy().to_string(),
        })
        .await?;

    info!("Daemon response: {}", response);

    Ok(())
}
