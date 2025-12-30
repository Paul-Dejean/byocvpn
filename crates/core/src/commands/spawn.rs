use std::os::unix::fs::PermissionsExt;

use tokio::{fs, io::AsyncWriteExt};

use crate::{
    cloud_provider::{CloudProvider, InstanceInfo, SpawnInstanceParams},
    config::{generate_client_config, get_wireguard_config_file_path},
    crypto::generate_keypair,
    error::{ComputeProvisioningError, Result},
};

pub async fn spawn_instance(provider: &dyn CloudProvider, region: &str) -> Result<InstanceInfo> {
    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let params = SpawnInstanceParams {
        region,
        server_private_key: &server_private_key,
        client_public_key: &client_public_key,
    };

    let instance = provider.spawn_instance(&params).await.map_err(|error| {
        ComputeProvisioningError::InstanceSpawnFailed {
            region_name: region.to_string(),
            reason: error.to_string(),
        }
    })?;

    println!("Spawned instance: {}", instance.id);

    let client_config = generate_client_config(
        &client_private_key,
        &server_public_key,
        &instance.public_ip_v4,
    )?;

    let provider_name = provider.get_provider_name();
    let wireguard_file_path =
        get_wireguard_config_file_path(&provider_name, region, &instance.id).await?;

    let mut file = fs::File::create(wireguard_file_path.clone()).await?;
    file.write_all(client_config.as_bytes()).await?;

    // Set permissions: rw------- (i.e., 0o600)
    let metadata = fs::metadata(wireguard_file_path.clone()).await?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(wireguard_file_path.clone(), perms).await?;

    let str_path = wireguard_file_path.to_str();
    if let Some(sp) = str_path {
        println!("Client config written to {}", sp);
    }

    Ok(instance)
}
