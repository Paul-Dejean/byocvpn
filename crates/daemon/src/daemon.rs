use byocvpn_core::{
    daemon_client::{DaemonCommand, DaemonResponse},
    error::{DaemonError, Result},
    ipc::IpcSocket,
};
use log::*;
use serde_json::Value;

use crate::{
    constants,
    vpn::{
        connect::connect_vpn, disconnect::disconnect_vpn, metrics::get_current_metrics,
        status::get_vpn_status,
    },
};

pub async fn run_daemon() -> Result<()> {
    env_logger::init();
    let socket_path = constants::socket_path();

    #[cfg(unix)]
    if let Some(socket_dir) = socket_path.parent() {
        debug!("Creating socket directory: {:?}", socket_dir);
        tokio::fs::create_dir_all(socket_dir)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to create socket directory: {}", error),
            })?;
    }

    let mut listener = IpcSocket::bind(socket_path.clone()).await?;

    info!(
        "Daemon listening on {} (pid: {})",
        socket_path.to_string_lossy(),
        std::process::id()
    );

    loop {
        let mut stream = listener.accept().await?;

        while let Ok(Some(line)) = stream.read_message().await {
            debug!("Daemon received: {line}");

            let response = match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(command) => handle_command(command).await,
                Err(error) => {
                    error!("Invalid command: {}", error);
                    DaemonResponse::Err(DaemonError::CommandFailed {
                        command: error.to_string(),
                    })
                }
            };

            match serde_json::to_string(&response) {
                Ok(json) => {
                    if stream.send_message(&json).await.is_err() {
                        error!("Failed to send response to client");
                    }
                }
                Err(error) => {
                    error!("Failed to serialize response: {}", error);
                }
            }
        }
    }
}

async fn handle_command(command: DaemonCommand) -> DaemonResponse {
    match command {
        DaemonCommand::Connect(params) => match connect_vpn(params).await {
            Ok(()) => DaemonResponse::Ok(Value::Null),
            Err(error) => {
                error!("Connect error: {}", error);
                DaemonResponse::Err(DaemonError::CommandFailed {
                    command: error.to_string(),
                })
            }
        },
        DaemonCommand::Disconnect => match disconnect_vpn().await {
            Ok(()) => DaemonResponse::Ok(Value::Null),
            Err(error) => {
                error!("Disconnect error: {}", error);
                DaemonResponse::Err(DaemonError::CommandFailed {
                    command: error.to_string(),
                })
            }
        },
        DaemonCommand::Status => match get_vpn_status().await {
            Ok(status) => match serde_json::to_value(&status) {
                Ok(value) => DaemonResponse::Ok(value),
                Err(error) => {
                    error!("Status serialization error: {}", error);
                    DaemonResponse::Err(DaemonError::CommandFailed {
                        command: error.to_string(),
                    })
                }
            },
            Err(error) => {
                error!("Status error: {}", error);
                DaemonResponse::Err(DaemonError::CommandFailed {
                    command: error.to_string(),
                })
            }
        },
        DaemonCommand::Stats => {
            let stats = get_current_metrics().await;
            match serde_json::to_value(&stats) {
                Ok(value) => DaemonResponse::Ok(value),
                Err(error) => {
                    error!("Stats serialization error: {}", error);
                    DaemonResponse::Err(DaemonError::CommandFailed {
                        command: error.to_string(),
                    })
                }
            }
        }
        DaemonCommand::HealthCheck => DaemonResponse::Ok(Value::Null),
    }
}
