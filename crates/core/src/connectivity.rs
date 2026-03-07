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

/// How many times to attempt the probe before giving up (~5 minutes total).
const PROBE_MAX_ATTEMPTS: u32 = 30;
/// Seconds to wait between probe attempts.
const PROBE_RETRY_DELAY_SECS: u64 = 10;
/// Seconds to wait for a single TCP connect to succeed.
const PROBE_CONNECT_TIMEOUT_SECS: u64 = 3;

/// Probe the WireGuard health endpoint on `{ip}:51820` (TCP) until the
/// instance reports `"active"` or the timeout is exhausted.
///
/// The health server is a `socat` process started by cloud-init after
/// `wg-quick@wg0` comes up. Each TCP connection receives the output of
/// `systemctl is-active wg-quick@wg0` — either `"active"` (success) or
/// `"inactive"`/`"failed"` (WireGuard not running).
pub async fn wait_until_ready(ip: &str) -> Result<()> {
    let addr = format!("{}:51820", ip);

    for attempt in 1..=PROBE_MAX_ATTEMPTS {
        eprintln!(
            "[probe] attempt {}/{} → {}",
            attempt, PROBE_MAX_ATTEMPTS, addr
        );

        let connect = timeout(
            TokioDuration::from_secs(PROBE_CONNECT_TIMEOUT_SECS),
            AsyncTcpStream::connect(&addr),
        )
        .await;

        match connect {
            Ok(Ok(mut stream)) => {
                let mut buf = Vec::new();
                let _ = timeout(TokioDuration::from_secs(3), stream.read_to_end(&mut buf)).await;

                let response = String::from_utf8_lossy(&buf).trim().to_string();
                eprintln!("[probe] response: {:?}", response);

                if response == "active" {
                    eprintln!("[probe] instance ready");
                    return Ok(());
                } else {
                    return Err(SystemError::ReadinessProbeFailed {
                        reason: format!("wg-quick@wg0 status: {}", response),
                    }
                    .into());
                }
            }
            Ok(Err(error)) => {
                eprintln!("[probe] connect error: {}", error);
            }
            Err(_) => {
                eprintln!("[probe] connect timed out");
            }
        }

        if attempt < PROBE_MAX_ATTEMPTS {
            sleep(TokioDuration::from_secs(PROBE_RETRY_DELAY_SECS)).await;
        }
    }

    Err(SystemError::ReadinessProbeTimedOut.into())
}

pub fn can_connect_ipv6() -> bool {
    // Google's IPv6 DNS server (UDP/53, but TCP test works fine)
    let addr: SocketAddr = "[2001:4860:4860::8888]:53"
        .parse()
        .expect("Ipv6 [2001:4860:4860::8888]:53 is invalid");

    // Try to open a TCP connection with a 2-second timeout
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
}
