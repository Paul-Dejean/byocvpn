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
    public_ip_v4: Option<String>,
    public_ip_v6: Option<String>,
) -> Result<()> {
    let provider_name = provider.get_provider_name();
    let wireguard_file_path =
        get_wireguard_config_file_path(&provider_name, region, instance_id).await?;

    info!("Sending connect command to daemon...");
    let response = daemon_client
        .send_command(DaemonCommand::Connect {
            config_path: wireguard_file_path.to_string_lossy().to_string(),
            region: region.to_string(),
            provider: provider_name.to_string(),
            public_ip_v4,
            public_ip_v6,
        })
        .await?;

    info!("Daemon response: {}", response);

    Ok(())
}
