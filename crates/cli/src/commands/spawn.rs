use std::os::unix::fs::PermissionsExt;

use byocvpn_core::{cloud_provider::CloudProvider, generate_client_config, generate_keypair};
use tokio::{fs, io::AsyncWriteExt};

pub async fn spawn_instance(aws: &dyn CloudProvider) -> Result<(), Box<dyn std::error::Error>> {
    let (client_private_key, client_public_key) = generate_keypair();
    let (server_private_key, server_public_key) = generate_keypair();

    let (instance_id, public_ip_v4, _public_ip_v6) = aws
        .spawn_instance(&server_private_key, &client_public_key)
        .await
        .expect("Failed to spawn instance");

    println!("Spawned instance: {}", instance_id);

    let client_config =
        generate_client_config(&client_private_key, &server_public_key, &public_ip_v4);

    let path = "wg0.conf";
    let mut file = fs::File::create(path).await?;
    file.write_all(client_config.as_bytes()).await?;

    // Set permissions: rw------- (i.e., 0o600)
    let metadata = fs::metadata(path).await?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms).await?;

    println!("Client config written to wg0.conf");
    Ok(())
}
