use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

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

                let response = String::from_utf8_lossy(&buffer).trim().to_string();
                debug!("[probe] response: {:?}", response);

                if response == "active" {
                    debug!("[probe] instance ready");
                    return Ok(());
                } else {
                    return Err(SystemError::ReadinessProbeFailed {
                        reason: format!("wg-quick@wg0 status: {}", response),
                    }
                    .into());
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

pub fn can_connect_ipv6() -> bool {
    let addr: SocketAddr = "[2001:4860:4860::8888]:53"
        .parse()
        .expect("Ipv6 [2001:4860:4860::8888]:53 is invalid");

    let available = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok();
    debug!(
        "[probe] IPv6 connectivity: {}",
        if available {
            "available"
        } else {
            "unavailable"
        }
    );
    available
}
