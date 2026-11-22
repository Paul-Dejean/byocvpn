use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter, Result as FmtResult},
    net::IpAddr,
    sync::Arc,
};

use boringtun::{
    noise::{Tunn, TunnResult, errors::WireGuardError},
    x25519::{PublicKey, StaticSecret},
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    time::{Duration, Instant, interval},
};
use tun_rs::AsyncDevice;

use crate::{
    cloud_provider::CloudProviderName,
    error::{ConfigurationError, Result, TunnelError},
};

pub const TUNNEL_MTU: u16 = 1280;

const RATE_HISTORY_LENGTH: usize = 10;
const PACKET_BUFFER_LENGTH: usize = 1500;
const TIMER_INTERVAL: Duration = Duration::from_millis(250);
const KEY_LENGTH: usize = 32;
const MILLISECONDS_PER_SECOND: u128 = 1000;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
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
    metrics: Arc<RwLock<TunnelMetrics>>,
}

impl Debug for Tunnel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.debug_struct("Tunnel").finish_non_exhaustive()
    }
}

impl Tunnel {
    pub fn new(
        tun: AsyncDevice,
        udp: UdpSocket,
        wg: Tunn,
        shutdown_rx: watch::Receiver<()>,
    ) -> Self {
        Self {
            tun,
            udp,
            wg,
            shutdown_rx,
            metrics: Arc::new(RwLock::new(TunnelMetrics::default())),
        }
    }

    pub fn get_metrics(&self) -> Arc<RwLock<TunnelMetrics>> {
        Arc::clone(&self.metrics)
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tun_buf = [0_u8; PACKET_BUFFER_LENGTH];
        let mut udp_buf = [0_u8; PACKET_BUFFER_LENGTH];
        let mut out_buf = [0_u8; PACKET_BUFFER_LENGTH];
        let mut timer_interval = interval(TIMER_INTERVAL);
        info!("[Tunnel] Starting tunnel...");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    info!("[Tunnel] Shutdown requested.");
                    break;
                }
                result = self.tun.recv(&mut tun_buf) => {
                    let received_length = result.map_err(TunnelError::io("read from the TUN device"))?;
                    #[expect(clippy::indexing_slicing, reason = "recv never returns more than the buffer length")]
                    let received_packet = &tun_buf[..received_length];
                    self.forward_outbound_packet(received_packet, &mut out_buf).await;
                }
                Ok((received_length, source_address)) = self.udp.recv_from(&mut udp_buf) => {
                    #[expect(clippy::indexing_slicing, reason = "recv_from never returns more than the buffer length")]
                    let received_packet = &udp_buf[..received_length];
                    self.forward_inbound_packet(source_address.ip(), received_packet, &mut out_buf).await?;
                }
                _ = timer_interval.tick() => {
                    self.handle_timer_tick(&mut out_buf).await;
                }
            }
        }

        info!("[Tunnel] Clean shutdown.");
        Ok(())
    }

    async fn forward_outbound_packet(&mut self, received_packet: &[u8], out_buf: &mut [u8]) {
        match self.wg.encapsulate(received_packet, out_buf) {
            TunnResult::WriteToNetwork(packet) => match self.udp.send(packet).await {
                Ok(sent) => self.record_sent(sent).await,
                Err(error) => warn!("[Tunnel] UDP send failed: {error}"),
            },
            TunnResult::Err(error) => error!("encapsulate error: {error:?}"),
            _ => {}
        }
    }

    async fn forward_inbound_packet(
        &mut self,
        source_ip: IpAddr,
        received_packet: &[u8],
        out_buf: &mut [u8],
    ) -> Result<()> {
        self.record_received(received_packet.len()).await;
        match self
            .wg
            .decapsulate(Some(source_ip), received_packet, out_buf)
        {
            TunnResult::WriteToTunnelV4(packet, _) | TunnResult::WriteToTunnelV6(packet, _) => {
                self.tun
                    .send(packet)
                    .await
                    .map_err(TunnelError::io("write to the TUN device"))?;
            }
            TunnResult::WriteToNetwork(packet) => {
                self.udp
                    .send(packet)
                    .await
                    .map_err(TunnelError::io("send over the UDP socket"))?;
                self.flush_queued_packets(out_buf).await?;
            }
            TunnResult::Done => {}
            TunnResult::Err(error) => error!("decapsulate error: {error:?}"),
        }
        Ok(())
    }

    async fn flush_queued_packets(&mut self, out_buf: &mut [u8]) -> Result<()> {
        while let TunnResult::WriteToNetwork(queued_packet) =
            self.wg.decapsulate(None, &[], out_buf)
        {
            self.udp
                .send(queued_packet)
                .await
                .map_err(TunnelError::io("send over the UDP socket"))?;
        }
        Ok(())
    }

    async fn handle_timer_tick(&mut self, out_buf: &mut [u8]) {
        match self.wg.update_timers(out_buf) {
            TunnResult::WriteToNetwork(packet) => {
                if let Err(error) = self.udp.send(packet).await {
                    warn!("[Tunnel] UDP send failed during timer update: {error}");
                }
            }
            TunnResult::Err(WireGuardError::ConnectionExpired) => {
                warn!("[Tunnel] WireGuard session expired, initiating new handshake.");
                self.initiate_handshake(out_buf).await;
            }
            TunnResult::Err(error) => error!("[Tunnel] Timer update error: {error:?}"),
            _ => {}
        }
    }

    async fn initiate_handshake(&mut self, out_buf: &mut [u8]) {
        match self.wg.format_handshake_initiation(out_buf, true) {
            TunnResult::WriteToNetwork(packet) => {
                if let Err(error) = self.udp.send(packet).await {
                    warn!("[Tunnel] Handshake initiation send failed: {error}");
                }
            }
            TunnResult::Err(error) => error!("[Tunnel] Handshake initiation failed: {error:?}"),
            _ => {}
        }
    }

    async fn record_sent(&self, byte_count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.bytes_sent += u64::try_from(byte_count).unwrap_or(u64::MAX);
        metrics.packets_sent += 1;
    }

    async fn record_received(&self, byte_count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.bytes_received += u64::try_from(byte_count).unwrap_or(u64::MAX);
        metrics.packets_received += 1;
    }
}

pub fn create_wireguard_tunnel(private_key: &[u8], public_key: &[u8]) -> Result<Tunn> {
    let private_key_bytes = parse_key_bytes(private_key, "private_key")?;
    let public_key_bytes = parse_key_bytes(public_key, "public_key")?;

    Ok(Tunn::new(
        StaticSecret::from(private_key_bytes),
        PublicKey::from(public_key_bytes),
        None,
        Some(25),
        0,
        None,
    ))
}

#[expect(
    clippy::map_err_ignore,
    reason = "TryFromSliceError carries nothing beyond the length mismatch"
)]
fn parse_key_bytes(key: &[u8], field: &'static str) -> Result<[u8; KEY_LENGTH]> {
    key.try_into().map_err(|_| {
        ConfigurationError::InvalidValue {
            field,
            reason: "key must be exactly 32 bytes",
        }
        .into()
    })
}

#[derive(Debug)]
pub struct TunnelRateTracker {
    last_metrics: TunnelMetrics,
    last_sampled_at: Instant,
    upload_history: VecDeque<u64>,
    download_history: VecDeque<u64>,
}

impl TunnelRateTracker {
    pub fn new() -> Self {
        Self {
            last_metrics: TunnelMetrics::default(),
            last_sampled_at: Instant::now(),
            upload_history: VecDeque::with_capacity(RATE_HISTORY_LENGTH),
            download_history: VecDeque::with_capacity(RATE_HISTORY_LENGTH),
        }
    }

    pub fn sample(&mut self, current_metrics: TunnelMetrics) -> TunnelMetrics {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sampled_at);

        let upload_rate_instant = calculate_instant_rate(
            current_metrics.bytes_sent,
            self.last_metrics.bytes_sent,
            elapsed,
        );
        let download_rate_instant = calculate_instant_rate(
            current_metrics.bytes_received,
            self.last_metrics.bytes_received,
            elapsed,
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

fn calculate_instant_rate(current_bytes: u64, previous_bytes: u64, elapsed: Duration) -> u64 {
    let elapsed_milliseconds = elapsed.as_millis();
    if elapsed_milliseconds == 0 {
        return 0;
    }
    let transferred = u128::from(current_bytes.saturating_sub(previous_bytes));
    let bytes_per_second = transferred * MILLISECONDS_PER_SECOND / elapsed_milliseconds;
    u64::try_from(bytes_per_second).unwrap_or(u64::MAX)
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
        let sample_count = u64::try_from(history.len()).unwrap_or(u64::MAX);
        history.iter().sum::<u64>() / sample_count
    }
}
