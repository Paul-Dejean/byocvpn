use byocvpn_core::{daemon_client::DaemonCommand, error::{DaemonError, Result}, ipc::IpcSocket};
use log::*;

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

    if let Some(socket_dir) = socket_path.parent() {
        tokio::fs::create_dir_all(socket_dir)
            .await
            .map_err(|error| DaemonError::SocketError {
                reason: format!("failed to create socket directory: {}", error),
            })?;
    }

    let listener = IpcSocket::bind(socket_path.clone()).await?;

    info!("Daemon listening on {}", socket_path.to_string_lossy());
    info!("process id: {}", std::process::id());

    loop {
        let mut stream = listener.accept().await?;

        while let Ok(Some(line)) = stream.read_message().await {
            info!("Daemon received: {line}");
            info!("process id: {}", std::process::id());
            match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(DaemonCommand::Connect { config_path }) => {
                    info!("Daemon received connect: {config_path}");
                    match connect_vpn(config_path).await {
                        Ok(_) => {
                            if stream.send_message("ok:connected").await.is_err() {
                                error!("Failed to send response to client");
                            }
                        }
                        Err(e) => {
                            error!("Connect error: {}", e);
                            if stream.send_message(&format!("err:{}", e)).await.is_err() {
                                error!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::Disconnect) => match disconnect_vpn().await {
                    Ok(_) => {
                        if stream.send_message("ok:disconnected").await.is_err() {
                            error!("Failed to send response to client");
                        }
                    }
                    Err(e) => {
                        error!("Disconnect error: {}", e);
                        if stream.send_message(&format!("err:{}", e)).await.is_err() {
                            error!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Status) => match get_vpn_status().await {
                    Ok(status) => match serde_json::to_string(&status) {
                        Ok(json) => {
                            if stream.send_message(&format!("ok:{}", json)).await.is_err() {
                                error!("Failed to send status response to client");
                            }
                        }
                        Err(e) => {
                            error!("Status serialization error: {}", e);
                            if stream.send_message(&format!("err:{}", e)).await.is_err() {
                                error!("Failed to send error response to client");
                            }
                        }
                    },
                    Err(e) => {
                        error!("Status error: {}", e);
                        if stream.send_message(&format!("err:{}", e)).await.is_err() {
                            error!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Stats) => {
                    let stats = get_current_metrics().await;
                    match serde_json::to_string(&stats) {
                        Ok(json) => {
                            if stream.send_message(&format!("ok:{}", json)).await.is_err() {
                                error!("Failed to send stats response to client");
                            }
                        }
                        Err(e) => {
                            error!("Stats serialization error: {}", e);
                            if stream.send_message(&format!("err:{}", e)).await.is_err() {
                                error!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::HealthCheck) => {
                    if stream.send_message("ok:healthy").await.is_err() {
                        error!("Failed to send health response to client");
                    }
                }

                Err(e) => {
                    error!("Invalid command: {}", e);
                    if stream.send_message(&format!("err:{}", e)).await.is_err() {
                        error!("Failed to send error response to client");
                    }
                }
            }
        }
    }
}
