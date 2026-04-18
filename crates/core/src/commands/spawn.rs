#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use log::*;
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams, SpawnStep,
        SpawnStepStatus,
    },
    config::{generate_client_config, get_wireguard_config_file_path},
    connectivity,
    error::{ComputeProvisioningError, ConfigurationError, Result},
};

pub async fn run_spawn_steps<F>(
    provider: &dyn CloudProvider,
    steps: &[SpawnStep],
    region: &str,
    client_private_key: &str,
    server_private_key: &str,
    client_public_key: &str,
    server_public_key: &str,
    on_step_progress: F,
) -> Result<InstanceInfo>
where
    F: Fn(&str, SpawnStepStatus, Option<String>),
{
    let mut spawned_instance: Option<InstanceInfo> = None;

    for step in steps {
        match step.id.as_str() {
            "launch" => {
                on_step_progress("launch", SpawnStepStatus::Running, None);
                match launch_instance(provider, region, server_private_key, client_public_key).await
                {
                    Ok(instance) => {
                        on_step_progress("launch", SpawnStepStatus::Completed, None);
                        let provider_name = provider.get_provider_name();
                        write_wireguard_config(
                            &provider_name,
                            region,
                            &instance,
                            client_private_key,
                            server_public_key,
                        )
                        .await?;
                        spawned_instance = Some(instance);
                    }
                    Err(error) => {
                        on_step_progress(
                            "launch",
                            SpawnStepStatus::Failed,
                            Some(error.to_string()),
                        );
                        return Err(error);
                    }
                }
            }
            "wireguard_ready" => {
                let instance_ip = spawned_instance
                    .as_ref()
                    .expect("launch step must precede wireguard_ready")
                    .public_ip_v4
                    .clone();
                on_step_progress("wireguard_ready", SpawnStepStatus::Running, None);
                if let Err(error) = connectivity::wait_until_ready(&instance_ip).await {
                    on_step_progress(
                        "wireguard_ready",
                        SpawnStepStatus::Failed,
                        Some(error.to_string()),
                    );
                    return Err(error);
                }
                on_step_progress("wireguard_ready", SpawnStepStatus::Completed, None);
            }
            step_id => {
                let step_id = step_id.to_string();
                on_step_progress(&step_id, SpawnStepStatus::Running, None);
                if let Err(error) = provider.run_spawn_step(&step_id, region).await {
                    on_step_progress(&step_id, SpawnStepStatus::Failed, Some(error.to_string()));
                    return Err(error);
                }
                on_step_progress(&step_id, SpawnStepStatus::Completed, None);
            }
        }
    }

    Ok(spawned_instance.expect("launch step must have run"))
}

pub async fn launch_instance(
    provider: &dyn CloudProvider,
    region: &str,
    server_private_key: &str,
    client_public_key: &str,
) -> Result<InstanceInfo> {
    let params = SpawnInstanceParams {
        region,
        server_private_key,
        client_public_key,
    };

    let instance = provider.spawn_instance(&params).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    info!("Spawned instance: {}", instance.id);
    Ok(instance)
}

pub async fn write_wireguard_config(
    provider_name: &CloudProviderName,
    region: &str,
    instance: &InstanceInfo,
    client_private_key: &str,
    server_public_key: &str,
) -> Result<()> {
    let client_config = generate_client_config(
        client_private_key,
        server_public_key,
        &instance.public_ip_v4,
    )?;

    let wireguard_file_path =
        get_wireguard_config_file_path(provider_name, region, &instance.id).await?;

    let mut file = fs::File::create(wireguard_file_path.clone())
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to create config file: {}", error),
        })?;
    file.write_all(client_config.as_bytes())
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to write config file: {}", error),
        })?;

    #[cfg(unix)]
    {
        let metadata = fs::metadata(wireguard_file_path.clone())
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to read config file metadata: {}", error),
            })?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(wireguard_file_path.clone(), perms)
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to set config file permissions: {}", error),
            })?;
    }

    if let Some(path_str) = wireguard_file_path.to_str() {
        info!("Client config written to {}", path_str);
    }

    Ok(())
}
