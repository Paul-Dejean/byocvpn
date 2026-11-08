use byocvpn_core::{
    daemon_client::{DaemonCommand, DaemonResponse},
    error::{DaemonError, Error, Result},
    ipc::{IpcSocket, socket_path},
};
use log::{debug, error, info};
use serde_json::Value;

use crate::vpn::{
    connect::connect_vpn, disconnect::disconnect_vpn, metrics::get_current_metrics,
    restore::try_restore_session, status::get_vpn_status,
};

pub async fn run_daemon() -> Result<()> {
    env_logger::init();
    let socket_path = socket_path();

    #[cfg(unix)]
    if let Some(socket_dir) = socket_path.parent() {
        debug!("Creating socket directory: {}", socket_dir.display());
        tokio::fs::create_dir_all(socket_dir)
            .await
            .map_err(DaemonError::Socket)?;
    }

    let mut listener = IpcSocket::bind(socket_path.clone()).await?;

    try_restore_session().await;

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
                Ok(command) => match handle_command(command).await {
                    Ok(value) => DaemonResponse::Ok(value),
                    Err(error) => {
                        error!("Command failed: {error}");
                        DaemonResponse::Err(error.to_payload())
                    }
                },
                Err(error) => {
                    error!("Invalid command: {error}");
                    DaemonResponse::Err(Error::from(error).to_payload())
                }
            };

            match serde_json::to_string(&response) {
                Ok(json) => {
                    if stream.send_message(&json).await.is_err() {
                        error!("Failed to send response to client");
                    }
                }
                Err(error) => {
                    error!("Failed to serialize response: {error}");
                }
            }
        }
    }
}

async fn handle_command(command: DaemonCommand) -> Result<Value> {
    match command {
        DaemonCommand::Connect(params) => {
            connect_vpn(*params).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Disconnect => {
            disconnect_vpn().await?;
            Ok(Value::Null)
        }
        DaemonCommand::Status => Ok(serde_json::to_value(get_vpn_status().await?)?),
        DaemonCommand::Stats => Ok(serde_json::to_value(get_current_metrics().await)?),
        DaemonCommand::HealthCheck => Ok(Value::Null),
    }
}
