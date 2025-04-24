use byocvpn_aws::AwsProvider;
use clap::{Parser, Subcommand};

mod commands {
    pub mod connect;
    pub mod disconnect;
    pub mod list;
    pub mod spawn;
    pub mod terminate;
}
#[derive(Parser)]
#[command(name = "byocvpn")]
#[command(about = "BYOC VPN CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Spawn,
    Terminate {
        #[arg(help = "The EC2 instance ID to terminate")]
        instance_id: String,
    },
    List,
    Connect,
    Disconnect,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let aws = AwsProvider::new()
        .await
        .expect("Failed to initialize AWS provider");

    match cli.command {
        Commands::Spawn => commands::spawn::spawn_instance(&aws).await?,
        Commands::Connect => commands::connect::connect().await?,
        Commands::Disconnect => commands::disconnect::disconnect().await?,
        Commands::Terminate { instance_id } => {
            commands::terminate::terminate_instance(&aws, &instance_id).await?
        }
        Commands::List => commands::list::list_instances(&aws).await?,
    }
    Ok(())
}
