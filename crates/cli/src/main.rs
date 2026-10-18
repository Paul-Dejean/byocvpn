mod connect;
mod error;
mod progress;
mod providers;
mod spawn;
mod terminate;

use byocvpn_core::cloud_provider::CloudProviderName;
use byocvpn_daemon::daemon_client::UnixDaemonClient;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;

use crate::{
    connect::{run_connect, run_disconnect},
    error::CliResult,
    providers::{describe_instance, list_instances_across_providers},
    spawn::spawn_new_server,
    terminate::run_terminate,
};

#[derive(Parser)]
#[command(name = "byocvpn")]
#[command(about = "BYOC VPN CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Spawn a new server without connecting to it")]
    Spawn {
        #[arg(short, long, help = "Cloud provider (asked if you have several)")]
        provider: Option<CloudProviderName>,

        #[arg(short, long, help = "Region to spawn in (asked if omitted)")]
        region: Option<String>,
    },
    Terminate {
        #[arg(help = "The instance ID to terminate (chosen from a list if omitted)")]
        instance_id: Option<String>,
    },
    List {
        #[arg(short, long, help = "Only list instances of this provider")]
        provider: Option<CloudProviderName>,

        #[arg(short, long, help = "Only list instances in this region")]
        region: Option<String>,
    },
    #[command(
        about = "Connect to a server, choosing from your instances or spawning a new one if no ID is given"
    )]
    Connect {
        #[arg(help = "The instance ID to connect to")]
        instance_id: Option<String>,
    },
    #[command(about = "Disconnect from the server you are currently connected to")]
    Disconnect,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(default_log_filter()))
        .format_timestamp(None)
        .format_target(false)
        .init();

    if let Err(error) = run(Cli::parse()).await {
        error!("{}", error);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> CliResult<()> {
    match cli.command {
        Commands::Spawn { provider, region } => {
            let instance = spawn_new_server(provider, region).await?;
            info!(
                "Instance ID: {}\nPublic IPv4: {}\nPublic IPv6: {}",
                instance.id, instance.public_ip_v4, instance.public_ip_v6
            );
        }
        Commands::Connect { instance_id } => {
            run_connect(&UnixDaemonClient, instance_id).await?;
        }
        Commands::Disconnect => {
            run_disconnect(&UnixDaemonClient, true).await?;
        }
        Commands::Terminate { instance_id } => {
            run_terminate(&UnixDaemonClient, instance_id).await?;
        }
        Commands::List { provider, region } => {
            let active_instances =
                list_instances_across_providers(provider, region.as_deref()).await?;
            if active_instances.is_empty() {
                info!("No active instances");
            }
            for instance in active_instances {
                info!("{}  {:?}", describe_instance(&instance), instance.state);
            }
        }
    }
    Ok(())
}

fn default_log_filter() -> &'static str {
    if cfg!(debug_assertions) {
        "info"
    } else {
        "warn,byocvpn_cli=info"
    }
}
