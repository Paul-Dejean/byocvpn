use tokio::fs;

use crate::{cloud_provider::CloudProvider, credentials::get_configs_path, error::Result};

pub async fn terminate_instance(provider: &dyn CloudProvider, instance_id: &str) -> Result<()> {
    provider
        .terminate_instance(&instance_id)
        .await
        .expect("Failed to terminate instance");
    let config_file_name = provider.get_config_file_name(&instance_id)?;
    let directory = get_configs_path().await?;
    let path = directory.join(config_file_name);
    if fs::metadata(&path).await.is_ok() {
        fs::remove_file(&path).await?;
    }
    println!("Terminated instance: {}", instance_id);
    Ok(())
}
