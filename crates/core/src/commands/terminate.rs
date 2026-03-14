use tokio::fs;

use crate::{
    cloud_provider::{CloudProvider, TerminateInstanceParams},
    config::get_wireguard_config_file_path,
    error::{ConfigurationError, Result},
};
use log::*;

pub async fn terminate_instance(
    provider: &dyn CloudProvider,
    region: &str,
    instance_id: &str,
) -> Result<()> {
    let params = TerminateInstanceParams {
        region,
        instance_id,
    };
    provider.terminate_instance(&params).await?;

    let provider_name = provider.get_provider_name();
    let wireguard_file_path =
        get_wireguard_config_file_path(&provider_name, region, instance_id).await?;

    if fs::metadata(&wireguard_file_path).await.is_ok() {
        fs::remove_file(&wireguard_file_path)
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to remove config file: {}", error),
            })?;
    }
    info!("Terminated instance: {}", instance_id);
    Ok(())
}
