use byocvpn_aws::{AwsCredentials, AwsProvider};
use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName},
    commands,
    connectivity::wait_until_ready,
    credentials::CredentialStore,
    crypto::generate_keypair,
    error::Result,
};
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use clap::{Parser, Subcommand};
use log::*;
#[derive(Parser)]
#[command(name = "byocvpn")]
#[command(about = "BYOC VPN CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Spawn {
        #[arg(short, long, help = "AWS region")]
        region: String,
    },
    Terminate {
        #[arg(help = "The EC2 instance ID to terminate")]
        instance_id: String,

        #[arg(short, long, help = "AWS region")]
        region: String,
    },
    List {
        #[arg(short, long, help = "AWS region")]
        region: Option<String>,
    },
    Connect {
        #[arg(help = "The EC2 instance ID to connect to")]
        instance_id: String,

        #[arg(short, long, help = "AWS region")]
        region: String,
    },
    Disconnect,
    Setup,
    EnableRegion {
        #[arg(short, long, help = "AWS region")]
        region: String,
    },
}

async fn create_cloud_provider(provider_name: CloudProviderName) -> Result<Box<dyn CloudProvider>> {
    match provider_name {
        CloudProviderName::Aws => {
            let store = CredentialStore::load().await?;
            Ok(Box::new(
                AwsProvider::new(AwsCredentials::from_store(&store)?.into()).await,
            ))
        }
        unsupported => {
            unimplemented!("CLI does not support provider: {}", unsupported)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Spawn { region } => {
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;
            commands::setup::setup(&*provider).await?;
            commands::setup::enable_region(&*provider, &region).await?;

            let (client_private_key, client_public_key) = generate_keypair();
            let (server_private_key, server_public_key) = generate_keypair();

            let instance = commands::spawn::launch_instance(
                &*provider,
                region.as_str(),
                &server_private_key,
                &client_public_key,
            )
            .await?;

            wait_until_ready(&instance.public_ip_v4).await?;

            let provider_name = provider.get_provider_name();
            commands::spawn::write_wireguard_config(
                &provider_name,
                region.as_str(),
                &instance,
                &client_private_key,
                &server_public_key,
            )
            .await?;

            info!(
                "Instance ID: {}\nPublic IPv4: {}\nPublic IPv6: {}",
                instance.id, instance.public_ip_v4, instance.public_ip_v6
            );
        }
        Commands::Connect {
            region,
            instance_id,
        } => {
            info!("Connecting to VPN...");
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;
            let daemon_client = UnixDaemonClient;

            commands::connect::connect(
                &*provider,
                &daemon_client,
                region.as_str(),
                instance_id.as_str(),
                None,
                None,
            )
            .await?;
            info!("Connected to VPN");
        }
        Commands::Disconnect => {
            info!("Disconnecting from VPN...");
            let daemon_client = UnixDaemonClient;
            commands::disconnect::disconnect(&daemon_client).await?;
            info!("Disconnected from VPN");
        }
        Commands::Terminate {
            region,
            instance_id,
        } => {
            info!("Terminating instance: {}", instance_id);
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;
            commands::terminate::terminate_instance(&*provider, &region, &instance_id).await?;
        }
        Commands::List { region } => {
            info!("Listing instances...");
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;
            let active_instances =
                commands::list::list_instances(&*provider, region.as_deref()).await?;
            info!(
                "{}",
                if active_instances.len() > 1 {
                    "Active Instances:"
                } else {
                    "No Active Instance"
                }
            );
            for instance in active_instances {
                info!("{:?}", instance);
            }
        }
        Commands::Setup => {
            info!("Setting up cloud provider...");
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;
            commands::setup::setup(&*provider).await?;
            info!("Cloud provider setup complete.");
        }
        Commands::EnableRegion { region } => {
            info!("Enabling region: {}", region);
            let provider = create_cloud_provider(CloudProviderName::Aws).await?;

            commands::setup::enable_region(&*provider, &region).await?;
            info!("Region enabled: {}", region);
        }
    }
    Ok(())
}
