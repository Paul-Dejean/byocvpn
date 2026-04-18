
use serde::Deserialize;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream as AsyncTcpStream,
    time::{Duration as TokioDuration, sleep, timeout},
};

use crate::error::{Result, SystemError};
use log::*;

const PROBE_MAX_ATTEMPTS: u32 = 30;

const PROBE_RETRY_DELAY_SECS: u64 = 10;

const PROBE_CONNECT_TIMEOUT_SECS: u64 = 3;

#[derive(Deserialize)]
struct ServerStatus {
    status: String,
}

pub async fn wait_until_ready(ip_address: &str) -> Result<()> {
    let address = format!("{}:51820", ip_address);

    for attempt in 1..=PROBE_MAX_ATTEMPTS {
        debug!(
            "[probe] attempt {}/{} → {}",
            attempt, PROBE_MAX_ATTEMPTS, address
        );

        let connect = timeout(
            TokioDuration::from_secs(PROBE_CONNECT_TIMEOUT_SECS),
            AsyncTcpStream::connect(&address),
        )
        .await;

        match connect {
            Ok(Ok(mut stream)) => {
                let mut buffer = Vec::new();
                let _ = timeout(TokioDuration::from_secs(3), stream.read_to_end(&mut buffer)).await;

                let raw = String::from_utf8_lossy(&buffer).trim().to_string();
                debug!("[probe] response: {:?}", raw);

                match serde_json::from_str::<ServerStatus>(&raw) {
                    Ok(ServerStatus { status }) if status == "ready" => {
                        debug!("[probe] instance ready");
                        return Ok(());
                    }
                    Ok(ServerStatus { status }) if status == "error" => {
                        return Err(SystemError::ReadinessProbeFailed {
                            reason: "WireGuard setup failed on the server".to_string(),
                        }
                        .into());
                    }
                    Ok(ServerStatus { status }) => {
                        debug!("[probe] server status: {}, retrying...", status);
                    }
                    Err(_) => {
                        debug!("[probe] unparseable response, retrying...");
                    }
                }
            }
            Ok(Err(error)) => {
                debug!("[probe] connect error: {}", error);
            }
            Err(_) => {
                debug!("[probe] connect timed out");
            }
        }

        if attempt < PROBE_MAX_ATTEMPTS {
            sleep(TokioDuration::from_secs(PROBE_RETRY_DELAY_SECS)).await;
        }
    }

    Err(SystemError::ReadinessProbeTimedOut.into())
}

