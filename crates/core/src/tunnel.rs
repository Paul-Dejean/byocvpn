use std::sync::Arc;

use boringtun::noise::{Tunn, TunnResult};
use serde::{Deserialize, Serialize};
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    time::{Duration, Instant},
};
use tun_rs::AsyncDevice;

use crate::error::{Result, SystemError};
use log::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub upload_rate: u64,
    pub download_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedInstance {
    pub instance_id: String,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnStatus {
    pub connected: bool,
    pub instance: Option<ConnectedInstance>,
    pub metrics: Option<TunnelMetrics>,
}

pub struct Tunnel {
    tun: AsyncDevice,
    udp: UdpSocket,
    wg: Tunn,
    shutdown_rx: watch::Receiver<()>,
    pub metrics: Arc<RwLock<TunnelMetrics>>,
}

impl Tunnel {
    pub fn new(
        tun: AsyncDevice,
        udp: UdpSocket,
        wg: Tunn,
        shutdown_rx: watch::Receiver<()>,
    ) -> Self {
        Tunnel {
            tun,
            udp,
            wg,
            shutdown_rx,
            metrics: Arc::new(RwLock::new(TunnelMetrics::default())),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tun_buf = [0u8; 1500];
        let mut udp_buf = [0u8; 1500];
        let mut out_buf = [0u8; 1500];
        let mut last_keepalive = Instant::now();
        info!("[Tunnel] Starting tunnel...");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    info!("[Tunnel] Shutdown requested.");
                    break;
                }

                Ok(n) = self.tun.recv(&mut tun_buf) => {

                    match self.wg.encapsulate(&tun_buf[..n], &mut out_buf) {
                        TunnResult::WriteToNetwork(packet) => {
                            if let Ok(sent) = self.udp.send(packet).await {
                                let mut metrics = self.metrics.write().await;
                                metrics.bytes_sent += sent as u64;
                                metrics.packets_sent += 1;
                            }
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(e) => {
                            error!("encapsulate error: {:?}", e);
                        },
                        _ => {}
                    }
                }

                Ok((n, src)) = self.udp.recv_from(&mut udp_buf) => {
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.bytes_received += n as u64;
                        metrics.packets_received += 1;
                    }

                    match self.wg.decapsulate(Some(src.ip()), &udp_buf[..n], &mut out_buf) {
                        TunnResult::WriteToTunnelV4(packet, _src_ip) => {
                            self.tun.send(packet).await.map_err(|error| SystemError::TunnelIoFailed { reason: error.to_string() })?;
                        },
                        TunnResult::WriteToTunnelV6(packet, _src_ip) => {
                            self.tun.send(packet).await.map_err(|error| SystemError::TunnelIoFailed { reason: error.to_string() })?;
                        },
                        TunnResult::WriteToNetwork(packet) => {
                            self.udp.send(packet).await.map_err(|error| SystemError::TunnelIoFailed { reason: error.to_string() })?;
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(e) => {
                            error!("decapsulate error: {:?}", e);
                        },
                    }
                }

                _ = tokio::time::sleep(Duration::from_secs(15)) => {
                    if last_keepalive.elapsed() >= Duration::from_secs(15) {

                        match self.wg.encapsulate(&[], &mut out_buf) {
                            TunnResult::WriteToNetwork(packet) => {
                                self.udp.send(packet).await.map_err(|error| SystemError::TunnelIoFailed { reason: error.to_string() })?;
                                last_keepalive = Instant::now();
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        info!("[Tunnel] Clean shutdown.");
        Ok(())
    }
}
