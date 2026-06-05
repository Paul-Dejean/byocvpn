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
    reason: Option<String>,
}

pub enum ProbeStatus {
    Ready,
    Installing,
    Error(String),
}

pub async fn probe_status(ip_address: &str) -> ProbeStatus {
    let address = format!("{}:51820", ip_address);

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
            match serde_json::from_str::<ServerStatus>(&raw) {
                Ok(ServerStatus { status, .. }) if status == "ready" => ProbeStatus::Ready,
                Ok(ServerStatus { status, reason }) if status == "error" => {
                    ProbeStatus::Error(reason.unwrap_or_else(|| "unknown error".to_string()))
                }
                _ => ProbeStatus::Installing,
            }
        }
        _ => ProbeStatus::Installing,
    }
}

pub async fn wait_until_ready(ip_address: &str) -> Result<()> {
    for attempt in 1..=PROBE_MAX_ATTEMPTS {
        debug!("[probe] attempt {}/{}", attempt, PROBE_MAX_ATTEMPTS);

        match probe_status(ip_address).await {
            ProbeStatus::Ready => {
                debug!("[probe] instance ready");
                return Ok(());
            }
            ProbeStatus::Error(reason) => {
                return Err(SystemError::ReadinessProbeFailed { reason }.into());
            }
            ProbeStatus::Installing => {
                debug!("[probe] not ready yet, retrying...");
            }
        }

        if attempt < PROBE_MAX_ATTEMPTS {
            sleep(TokioDuration::from_secs(PROBE_RETRY_DELAY_SECS)).await;
        }
    }

    Err(SystemError::ReadinessProbeTimedOut.into())
}
