use std::sync::Arc;

use boringtun::noise::{Tunn, TunnResult, errors::WireGuardError};
use serde::{Deserialize, Serialize};

use crate::cloud_provider::CloudProviderName;
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    time::{Duration, interval},
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
    pub region: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnStatus {
    pub connected: bool,
    pub instance: Option<ConnectedInstance>,
    pub metrics: Option<TunnelMetrics>,
    pub connected_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_error: Option<String>,
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
        let mut timer_interval = interval(Duration::from_millis(250));
        info!("[Tunnel] Starting tunnel...");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    info!("[Tunnel] Shutdown requested.");
                    break;
                }

                result = self.tun.recv(&mut tun_buf) => {
                    match result {
                        Ok(n) => {
                            match self.wg.encapsulate(&tun_buf[..n], &mut out_buf) {
                                TunnResult::WriteToNetwork(packet) => {
                                    match self.udp.send(packet).await {
                                        Ok(sent) => {
                                            let mut metrics = self.metrics.write().await;
                                            metrics.bytes_sent += sent as u64;
                                            metrics.packets_sent += 1;
                                        }
                                        Err(error) => {
                                            warn!("[Tunnel] UDP send failed: {}", error);
                                        }
                                    }
                                },
                                TunnResult::Done => {},
                                TunnResult::Err(error) => {
                                    error!("encapsulate error: {:?}", error);
                                },
                                _ => {}
                            }
                        }
                        Err(error) => {
                            error!("[Tunnel] TUN device read failed: {}", error);
                            return Err(SystemError::TunnelIoFailed { reason: error.to_string() }.into());
                        }
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
                            while let TunnResult::WriteToNetwork(queued_packet) = self.wg.decapsulate(None, &[], &mut out_buf) {
                                self.udp.send(queued_packet).await.map_err(|error| SystemError::TunnelIoFailed { reason: error.to_string() })?;
                            }
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(error) => {
                            error!("decapsulate error: {:?}", error);
                        },
                    }
                }

                _ = timer_interval.tick() => {
                    match self.wg.update_timers(&mut out_buf) {
                        TunnResult::WriteToNetwork(packet) => {
                            if let Err(error) = self.udp.send(packet).await {
                                warn!("[Tunnel] UDP send failed during timer update: {}", error);
                            }
                        },
                        TunnResult::Err(WireGuardError::ConnectionExpired) => {
                            warn!("[Tunnel] WireGuard session expired, initiating new handshake.");
                            match self.wg.format_handshake_initiation(&mut out_buf, true) {
                                TunnResult::WriteToNetwork(packet) => {
                                    if let Err(error) = self.udp.send(packet).await {
                                        warn!("[Tunnel] Handshake initiation send failed: {}", error);
                                    }
                                },
                                TunnResult::Err(error) => {
                                    error!("[Tunnel] Handshake initiation failed: {:?}", error);
                                },
                                _ => {}
                            }
                        },
                        TunnResult::Err(error) => {
                            error!("[Tunnel] Timer update error: {:?}", error);
                        },
                        _ => {}
                    }
                }
            }
        }

        info!("[Tunnel] Clean shutdown.");
        Ok(())
    }
}
