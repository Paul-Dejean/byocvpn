use std::{collections::VecDeque, sync::Arc};

use boringtun::{
    noise::{Tunn, TunnResult, errors::WireGuardError},
    x25519::{PublicKey, StaticSecret},
};
use serde::{Deserialize, Serialize};

use crate::cloud_provider::CloudProviderName;
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    time::{Duration, Instant, interval},
};
use tun_rs::AsyncDevice;

use crate::error::{ConfigurationError, Result, SystemError};
use log::*;

pub const TUNNEL_MTU: u16 = 1280;

const RATE_HISTORY_LENGTH: usize = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TunnelMetrics {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_sent: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub bytes_received: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub packets_sent: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub packets_received: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub upload_rate: u64,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub download_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ConnectedInstance {
    pub instance_id: String,
    pub public_ip_v4: Option<String>,
    pub public_ip_v6: Option<String>,
    pub region: String,
    pub provider: CloudProviderName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VpnStatus {
    pub connected: bool,
    pub instance: Option<ConnectedInstance>,
    pub metrics: Option<TunnelMetrics>,
    #[cfg_attr(feature = "specta", specta(type = Option<u32>))]
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

pub fn create_wireguard_tunnel(private_key: Vec<u8>, public_key: Vec<u8>) -> Result<Tunn> {
    let private_key_bytes: [u8; 32] =
        private_key
            .as_slice()
            .try_into()
            .map_err(|_| ConfigurationError::InvalidValue {
                field: "private_key".to_string(),
                reason: "Private key must be exactly 32 bytes".to_string(),
            })?;
    let public_key_bytes: [u8; 32] =
        public_key
            .as_slice()
            .try_into()
            .map_err(|_| ConfigurationError::InvalidValue {
                field: "public_key".to_string(),
                reason: "Public key must be exactly 32 bytes".to_string(),
            })?;

    Tunn::new(
        StaticSecret::from(private_key_bytes),
        PublicKey::from(public_key_bytes),
        None,
        Some(25),
        0,
        None,
    )
    .map_err(|error| {
        ConfigurationError::TunnelConfiguration {
            reason: format!("Failed to create WireGuard tunnel: {:?}", error),
        }
        .into()
    })
}

pub struct TunnelRateTracker {
    last_metrics: TunnelMetrics,
    last_sampled_at: Instant,
    upload_history: VecDeque<u64>,
    download_history: VecDeque<u64>,
}

impl TunnelRateTracker {
    pub fn new() -> Self {
        TunnelRateTracker {
            last_metrics: TunnelMetrics::default(),
            last_sampled_at: Instant::now(),
            upload_history: VecDeque::with_capacity(RATE_HISTORY_LENGTH),
            download_history: VecDeque::with_capacity(RATE_HISTORY_LENGTH),
        }
    }

    pub fn sample(&mut self, current_metrics: TunnelMetrics) -> TunnelMetrics {
        let now = Instant::now();
        let elapsed_seconds = now.duration_since(self.last_sampled_at).as_secs_f64();

        let upload_rate_instant = calculate_instant_rate(
            current_metrics.bytes_sent,
            self.last_metrics.bytes_sent,
            elapsed_seconds,
        );
        let download_rate_instant = calculate_instant_rate(
            current_metrics.bytes_received,
            self.last_metrics.bytes_received,
            elapsed_seconds,
        );

        push_rate_sample(&mut self.upload_history, upload_rate_instant);
        push_rate_sample(&mut self.download_history, download_rate_instant);

        let snapshot = TunnelMetrics {
            bytes_sent: current_metrics.bytes_sent,
            bytes_received: current_metrics.bytes_received,
            packets_sent: current_metrics.packets_sent,
            packets_received: current_metrics.packets_received,
            upload_rate: calculate_average_rate(&self.upload_history),
            download_rate: calculate_average_rate(&self.download_history),
        };

        self.last_metrics = current_metrics;
        self.last_sampled_at = now;
        snapshot
    }
}

impl Default for TunnelRateTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_instant_rate(current_bytes: u64, previous_bytes: u64, elapsed_seconds: f64) -> u64 {
    if elapsed_seconds > 0.0 {
        (current_bytes.saturating_sub(previous_bytes) as f64 / elapsed_seconds) as u64
    } else {
        0
    }
}

fn push_rate_sample(history: &mut VecDeque<u64>, rate: u64) {
    history.push_back(rate);
    if history.len() > RATE_HISTORY_LENGTH {
        history.pop_front();
    }
}

fn calculate_average_rate(history: &VecDeque<u64>) -> u64 {
    if history.is_empty() {
        0
    } else {
        history.iter().sum::<u64>() / history.len() as u64
    }
}
