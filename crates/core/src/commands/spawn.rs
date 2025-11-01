use std::os::unix::fs::PermissionsExt;

use tokio::{fs, io::AsyncWriteExt};

use crate::{
    cloud_provider::CloudProvider, generate_client_config, generate_keypair, get_configs_path,
};

pub async fn spawn_instance(
    provider: &dyn CloudProvider,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let (instance_id, public_ip_v4, _public_ip_v6) = provider
        .spawn_instance(&server_private_key, &client_public_key)
        .await
        .expect("Failed to spawn instance");

    println!("Spawned instance: {}", instance_id);

    let client_config =
        generate_client_config(&client_private_key, &server_public_key, &public_ip_v4);

    let directory = get_configs_path().await?;
    let file_name = provider.get_config_file_name(&instance_id)?;
    let path = directory.join(file_name);
    let mut file = fs::File::create(path.clone()).await?;
    file.write_all(client_config.as_bytes()).await?;

    // Set permissions: rw------- (i.e., 0o600)
    let metadata = fs::metadata(path.clone()).await?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path.clone(), perms).await?;

    println!(
        "Client config written to {}",
        path.clone().to_str().unwrap()
    );
    Ok((instance_id, public_ip_v4, client_config))
}
