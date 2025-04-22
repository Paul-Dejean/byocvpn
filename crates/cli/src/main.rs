use byocvpn_aws::AwsProvider;
use byocvpn_core::cloud_provider::CloudProvider;
use byocvpn_core::generate_client_config;
use byocvpn_core::generate_keypair;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(name = "byocvpn")]
#[command(about = "BYOC VPN CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Spawn a new EC2 VPN instance
    Spawn,
    /// Terminate a VPN instance by ID
    Terminate {
        #[arg(help = "The EC2 instance ID to terminate")]
        instance_id: String,
    },
    /// List active VPN instances (TODO)
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let aws = AwsProvider::new().await.unwrap();

    match cli.command {
        Commands::Spawn => {
            let (client_private_key, client_public_key) = generate_keypair();
            let (server_private_key, server_public_key) = generate_keypair();

            let (instance_id, public_ip) = aws
                .spawn_instance(&server_private_key, &client_public_key)
                .await
                .unwrap();
            println!("Spawned instance: {}", instance_id);
            let output = aws.get_console_output(&instance_id).await.unwrap();
            println!("{:?}", output);
            let client_config =
                generate_client_config(&client_private_key, &server_public_key, &public_ip);
            fs::write("wg0.conf", client_config).expect("Failed to write client config");
            println!("Client config written to ./wg0.conf");
        }
        Commands::Terminate { instance_id } => {
            aws.terminate_instance(&instance_id).await.unwrap();
            println!("Terminated instance: {}", instance_id);
        }
        Commands::List => {
            // placeholder
            let instances = aws.list_instances().await.unwrap();
            println!("Active instances: {:?}", instances);
        }
    }
}
