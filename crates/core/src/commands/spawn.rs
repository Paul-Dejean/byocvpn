use std::os::unix::fs::PermissionsExt;

use tokio::{fs, io::AsyncWriteExt};

use crate::{
    cloud_provider::{CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams},
    config::{generate_client_config, get_wireguard_config_file_path},
    error::{ComputeProvisioningError, Result},
};

/// Call the cloud provider's spawn API and return `InstanceInfo` as soon as
/// the instance has a public IP address.
///
/// WireGuard may not be running yet — call
/// [`crate::connectivity::wait_until_ready`] afterwards to probe the health
/// endpoint before writing the client config.
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

    println!("Spawned instance: {}", instance.id);
    Ok(instance)
}

/// Generate the WireGuard client config and write it to the standard path
/// with `0o600` permissions.
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

    let mut file = fs::File::create(wireguard_file_path.clone()).await?;
    file.write_all(client_config.as_bytes()).await?;

    let metadata = fs::metadata(wireguard_file_path.clone()).await?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(wireguard_file_path.clone(), perms).await?;

    if let Some(path_str) = wireguard_file_path.to_str() {
        println!("Client config written to {}", path_str);
    }

    Ok(())
}
