use byocvpn_aws::{AwsProvider, provider::AwsProviderConfig};
use byocvpn_core::{cloud_provider::CloudProvider, commands};
use clap::{Parser, Subcommand};

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
        region: String,
    },
    Connect {
        #[arg(help = "The EC2 instance ID to connect to")]
        instance_id: String,

        #[arg(short, long, help = "AWS region")]
        region: String,
    },
    Disconnect,
    SpawnDaemon,
    Setup,
    EnableRegion {
        #[arg(short, long, help = "AWS region")]
        region: String,
    },
}

async fn create_cloud_provider(
    cloud_provider_name: &str,
    region: Option<String>,
) -> Result<Box<dyn CloudProvider>, Box<dyn std::error::Error>> {
    match cloud_provider_name {
        "aws" => {
            // Get stored credentials
            let credentials = byocvpn_core::get_credentials().await?;

            // Create AWS provider config
            let config = AwsProviderConfig {
                region,
                access_key_id: Some(credentials.access_key.clone()),
                secret_access_key: Some(credentials.secret_access_key.clone()),
            };
            let cloud_provider = AwsProvider::new(&config).await?;

            Ok(Box::new(cloud_provider))
        }
        _ => return Err("Unsupported cloud provider".into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Spawn { region } => {
            let provider = create_cloud_provider("aws", Some(region)).await?;
            let (instance_id, ip_v4, ip_v6) = commands::spawn::spawn_instance(&*provider).await?;

            println!(
                "Instance ID: {}\nPublic IPv4: {}\nPublic IPv6: {}",
                instance_id, ip_v4, ip_v6
            );
        }
        Commands::Connect {
            region,
            instance_id,
        } => {
            println!("Connecting to VPN...");
            let provider = create_cloud_provider("aws", Some(region)).await?;
            commands::connect::connect(&*provider, instance_id).await?;
            println!("Connected to VPN");
        }
        Commands::Disconnect => {
            println!("Disconnecting from VPN...");
            commands::disconnect::disconnect().await?;
            println!("Disconnected from VPN");
        }
        Commands::Terminate {
            region,
            instance_id,
        } => {
            println!("Terminating instance: {}", instance_id);
            let provider = create_cloud_provider("aws", Some(region)).await?;
            commands::terminate::terminate_instance(&*provider, &instance_id).await?;
        }
        Commands::List { region } => {
            println!("Listing instances...");
            let provider = create_cloud_provider("aws", Some(region)).await?;
            let active_instances = commands::list::list_instances(&*provider).await?;
            println!(
                "{}",
                if active_instances.len() > 1 {
                    "Active Instances:"
                } else {
                    "No Active Instance"
                }
            );
            for instance in active_instances {
                println!("{:?}", instance);
            }
        }
        Commands::SpawnDaemon => {
            println!("Spawning daemon...");
            commands::spawn_daemon::spawn_daemon().await?;
            println!("Daemon spawned.");
        }
        Commands::Setup => {
            println!("Setting up cloud provider...");
            let provider = create_cloud_provider("aws", None).await?;
            commands::setup::setup(&*provider).await?;
            println!("Cloud provider setup complete.");
        }
        Commands::EnableRegion { region } => {
            println!("Enabling region: {}", region);
            let provider = create_cloud_provider("aws", Some(region.clone())).await?;

            commands::setup::enable_region(&*provider, &region).await?;
            println!("Region enabled: {}", region);
        }
    }
    Ok(())
}
