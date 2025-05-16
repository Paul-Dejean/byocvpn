use std::fs;

use byocvpn_core::{cloud_provider::CloudProvider, generate_client_config, generate_keypair};

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
    fs::write("wg0.conf", client_config).expect("Failed to write client config");
    println!("Client config written to ./wg0.conf");
    Ok(())
}
