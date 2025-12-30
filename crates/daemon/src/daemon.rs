use byocvpn_core::{daemon_client::DaemonCommand, error::Result, ipc::IpcSocket};

use crate::{
    constants,
    vpn::{
        connect::connect_vpn, disconnect::disconnect_vpn, metrics::get_current_metrics,
        status::get_vpn_status,
    },
};

pub async fn run_daemon() -> Result<()> {
    let socket_path = constants::socket_path();

    let listener = IpcSocket::bind(socket_path.clone()).await?;

    println!("Daemon listening on {}", socket_path.to_string_lossy());
    println!("process id: {}", std::process::id());

    loop {
        let mut stream = listener.accept().await?;

        while let Ok(Some(line)) = stream.read_message().await {
            println!("Daemon received: {line}");
            println!("process id: {}", std::process::id());
            match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(DaemonCommand::Connect { config_path }) => {
                    println!("Daemon received connect: {config_path}");
                    match connect_vpn(config_path).await {
                        Ok(_) => {
                            if stream.send_message("Connected!").await.is_err() {
                                eprintln!("Failed to send response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Connect error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message(&error_msg).await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::Disconnect) => match disconnect_vpn().await {
                    Ok(_) => {
                        if stream.send_message("Disconnected.").await.is_err() {
                            eprintln!("Failed to send response to client");
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Disconnect error: {}", e);
                        eprintln!("{}", error_msg);
                        if stream.send_message(&error_msg).await.is_err() {
                            eprintln!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Status) => match get_vpn_status().await {
                    Ok(status) => match serde_json::to_string(&status) {
                        Ok(json) => {
                            if stream.send_message(&json).await.is_err() {
                                eprintln!("Failed to send status response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Status serialization error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message(&error_msg).await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Status error: {}", e);
                        eprintln!("{}", error_msg);
                        if stream.send_message(&error_msg).await.is_err() {
                            eprintln!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Stats) => {
                    let stats = get_current_metrics().await;
                    match serde_json::to_string(&stats) {
                        Ok(response) => {
                            if stream.send_message(&response).await.is_err() {
                                eprintln!("Failed to send stats response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Stats serialization error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message("null").await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::HealthCheck) => {
                    if stream.send_message("healthy").await.is_err() {
                        eprintln!("Failed to send health response to client");
                    }
                }

                Err(e) => {
                    let error_msg = format!("Invalid command: {}", e);
                    eprintln!("{}", error_msg);
                    if stream.send_message(&error_msg).await.is_err() {
                        eprintln!("Failed to send error response to client");
                    }
                }
            }
        }
    }
}
