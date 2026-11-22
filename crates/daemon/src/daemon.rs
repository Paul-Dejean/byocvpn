use std::process::id as process_id;

#[cfg(unix)]
use byocvpn_core::error::DaemonError;
use byocvpn_core::{
    daemon_client::{DaemonCommand, DaemonResponse},
    error::{Error, Result},
    ipc::{IpcSocket, socket_path},
    logging::{LoggingFormat, initialize_logging},
};
use serde_json::Value;
#[cfg(unix)]
use tokio::fs::create_dir_all;
use tracing::{Instrument, debug, debug_span, error, info, warn};

use crate::vpn::{
    connect::connect_vpn, disconnect::disconnect_vpn, metrics::get_current_metrics,
    restore::try_restore_session, status::get_vpn_status,
};

pub async fn run_daemon() -> Result<()> {
    initialize_logging("info", LoggingFormat::DETAILED);
    let socket_path = socket_path();

    #[cfg(unix)]
    if let Some(socket_dir) = socket_path.parent() {
        debug!("creating socket directory: {}", socket_dir.display());
        create_dir_all(socket_dir)
            .await
            .map_err(DaemonError::Socket)?;
    }

    let mut listener = IpcSocket::bind(socket_path.clone()).await?;

    try_restore_session().await;

    info!(socket = %socket_path.to_string_lossy(), pid = process_id(), "daemon listening");

    loop {
        let mut stream = listener.accept().await?;

        while let Ok(Some(line)) = stream.read_message().await {
            let response = match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(command) => {
                    let span = debug_span!("command", kind = describe_command(&command));
                    match handle_command(command).instrument(span).await {
                        Ok(value) => DaemonResponse::Ok(value),
                        Err(error) => {
                            warn!(error = %error.describe(), "command failed");
                            DaemonResponse::Err(error.to_payload())
                        }
                    }
                }
                Err(error) => {
                    warn!(%error, "invalid command");
                    DaemonResponse::Err(Error::from(error).to_payload())
                }
            };

            match serde_json::to_string(&response) {
                Ok(json) => {
                    if stream.send_message(&json).await.is_err() {
                        warn!("failed to send response to client");
                    }
                }
                Err(error) => {
                    error!(%error, "failed to serialize response");
                }
            }
        }
    }
}

const fn describe_command(command: &DaemonCommand) -> &'static str {
    match command {
        DaemonCommand::Connect(_) => "connect",
        DaemonCommand::Disconnect => "disconnect",
        DaemonCommand::Status => "status",
        DaemonCommand::Stats => "stats",
        DaemonCommand::HealthCheck => "health_check",
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
