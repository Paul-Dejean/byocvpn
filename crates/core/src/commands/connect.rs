use crate::{
    cloud_provider::CloudProvider,
    config::get_wireguard_config_file_path,
    daemon_client::{DaemonClient, DaemonCommand},
    error::Result,
    wireguard_config::parse_wireguard_config,
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

    let wireguard_config = parse_wireguard_config(&wireguard_file_path.to_string_lossy()).await?;

    info!("Sending connect command to daemon...");
    let response = daemon_client
        .send_command(DaemonCommand::Connect {
            instance_id: wireguard_config.instance_id,
            private_key: wireguard_config.private_key,
            public_key: wireguard_config.public_key,
            endpoint: wireguard_config.endpoint,
            ipv4: wireguard_config.ipv4,
            ipv6: wireguard_config.ipv6,
            dns_servers: wireguard_config.dns_servers,
            region: region.to_string(),
            provider: provider_name.to_string(),
            public_ip_v4,
            public_ip_v6,
        })
        .await?;

    info!("Daemon response: {}", response);

    Ok(())
}
