use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket as StandardUdpSocket},
    os::fd::{AsRawFd, RawFd},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use byocvpn_core::{
    daemon_client::{DaemonClient, DaemonCommand, VpnConnectParams},
    error::{ConfigurationError, DaemonError, Error, Result, SystemError},
    tunnel::{
        ConnectedInstance, TUNNEL_MTU, Tunnel, TunnelMetrics, TunnelRateTracker, VpnStatus,
        create_wireguard_tunnel,
    },
};
use log::*;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Wry, async_runtime, plugin::mobile::PluginInvokeError};
use tauri_plugin_android_vpn::{AndroidVpn, AndroidVpnExt, TunnelInterfaceRequest};
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    task::JoinHandle,
    time::interval,
};
use tun_rs::AsyncDevice;

use crate::{ledger_store::LedgerStore, vpn_backend::publish_vpn_status};

const VPN_SESSION_NAME: &str = "ByocVPN";
const STATUS_INTERVAL: Duration = Duration::from_secs(1);
const METRICS_PERSIST_INTERVAL: Duration = Duration::from_secs(60);

static ACTIVE_TUNNEL: Mutex<Option<ActiveTunnel>> = Mutex::new(None);
static STATUS_STREAM_GENERATION: AtomicU64 = AtomicU64::new(0);

struct ActiveTunnel {
    shutdown: watch::Sender<()>,
    task: JoinHandle<()>,
    metrics: Arc<RwLock<TunnelMetrics>>,
    instance: ConnectedInstance,
    connected_at: SystemTime,
}

struct ActiveTunnelSnapshot {
    is_finished: bool,
    instance: ConnectedInstance,
    metrics: Arc<RwLock<TunnelMetrics>>,
    connected_at: SystemTime,
}

pub struct AndroidDaemonClient {
    vpn_service: AndroidVpn<Wry>,
}

pub fn create_daemon_client(app_handle: &AppHandle) -> AndroidDaemonClient {
    AndroidDaemonClient {
        vpn_service: app_handle.android_vpn().clone(),
    }
}

#[async_trait]
impl DaemonClient for AndroidDaemonClient {
    async fn send_command(&self, command: DaemonCommand) -> Result<Value> {
        match command {
            DaemonCommand::Connect(connect_params) => {
                connect_tunnel(&self.vpn_service, connect_params).await?;
                Ok(Value::Null)
            }
            DaemonCommand::Disconnect => {
                disconnect_tunnel(&self.vpn_service).await?;
                Ok(Value::Null)
            }
            DaemonCommand::Status => {
                serialize_response(&read_tunnel_status(&self.vpn_service).await?)
            }
            DaemonCommand::Stats => serialize_response(&read_tunnel_metrics().await?),
            DaemonCommand::HealthCheck => Ok(Value::Null),
        }
    }

    async fn is_daemon_running(&self) -> bool {
        true
    }
}

pub async fn start_status_stream(
    app_handle: &AppHandle,
    connected_instance: ConnectedInstance,
    connected_at: Option<u64>,
) -> Result<()> {
    start_status_subscription(app_handle, connected_instance, connected_at).await
}

pub async fn start_status_subscription(
    app_handle: &AppHandle,
    connected_instance: ConnectedInstance,
    _connected_at: Option<u64>,
) -> Result<()> {
    let generation = STATUS_STREAM_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let stream_handle = app_handle.clone();
    async_runtime::spawn(async move {
        run_status_stream(stream_handle, connected_instance, generation).await;
    });
    Ok(())
}

pub async fn stop_status_stream() -> Result<()> {
    STATUS_STREAM_GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn connect_tunnel(
    vpn_service: &AndroidVpn<Wry>,
    connect_params: VpnConnectParams,
) -> Result<()> {
    if is_tunnel_running()? {
        error!("Connect requested but a tunnel is already running.");
        return Err(ConfigurationError::TunnelConfiguration {
            reason: "Tunnel already running".to_string(),
        }
        .into());
    }

    info!(
        "Connecting VPN: instance={}, region={}, provider={}",
        connect_params.instance_id, connect_params.region, connect_params.provider
    );

    vpn_service
        .request_permission()
        .await
        .map_err(convert_plugin_error)?;

    let tunnel_file_descriptor = vpn_service
        .establish_tunnel_interface(TunnelInterfaceRequest {
            session_name: VPN_SESSION_NAME.to_string(),
            addresses: vec![
                connect_params.private_ipv4.to_string(),
                connect_params.private_ipv6.to_string(),
            ],
            dns_servers: connect_params.dns_servers.clone(),
            mtu: TUNNEL_MTU,
        })
        .await
        .map_err(convert_plugin_error)?;

    match start_tunnel(vpn_service, tunnel_file_descriptor, connect_params).await {
        Ok(active_tunnel) => {
            *lock_active_tunnel()? = Some(active_tunnel);
            info!("VPN setup complete.");
            Ok(())
        }
        Err(error) => {
            if let Err(stop_error) = vpn_service.stop().await {
                warn!("Failed to stop the VPN service after a failed connect: {stop_error}");
            }
            Err(error)
        }
    }
}

async fn disconnect_tunnel(vpn_service: &AndroidVpn<Wry>) -> Result<()> {
    info!("[VPN Disconnect] Disconnecting VPN tunnel...");
    let active_tunnel = lock_active_tunnel()?.take();

    match active_tunnel {
        Some(active_tunnel) => {
            if active_tunnel.shutdown.send(()).is_err() {
                debug!("[VPN Disconnect] Tunnel task already stopped.");
            }
            if let Err(error) = active_tunnel.task.await {
                warn!("[VPN Disconnect] Tunnel task panicked while shutting down: {error:?}");
            }
        }
        None => warn!("[VPN Disconnect] No active tunnel in memory."),
    }

    vpn_service.stop().await.map_err(convert_plugin_error)?;
    info!("[VPN Disconnect] VPN disconnected.");
    Ok(())
}

async fn read_tunnel_status(vpn_service: &AndroidVpn<Wry>) -> Result<VpnStatus> {
    let Some(snapshot) = read_active_tunnel_snapshot()? else {
        return Ok(build_disconnected_status());
    };

    let is_service_running = match vpn_service.is_running().await {
        Ok(is_running) => is_running,
        Err(error) => {
            warn!("Failed to read the VPN service status: {error}");
            true
        }
    };

    if snapshot.is_finished || !is_service_running {
        return Ok(build_disconnected_status());
    }

    let connected_at = snapshot
        .connected_at
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs());

    Ok(VpnStatus {
        connected: true,
        instance: Some(snapshot.instance),
        metrics: Some(snapshot.metrics.read().await.clone()),
        connected_at,
        connection_error: None,
    })
}

async fn read_tunnel_metrics() -> Result<Option<TunnelMetrics>> {
    match read_active_tunnel_snapshot()? {
        Some(snapshot) => Ok(Some(snapshot.metrics.read().await.clone())),
        None => Ok(None),
    }
}

async fn run_status_stream(
    app_handle: AppHandle,
    connected_instance: ConnectedInstance,
    generation: u64,
) {
    let vpn_service = app_handle.android_vpn().clone();
    let mut ticker = interval(STATUS_INTERVAL);
    let mut rate_tracker = TunnelRateTracker::new();
    let mut last_persisted_at: Option<Instant> = None;

    loop {
        ticker.tick().await;
        if !is_current_status_stream(generation) {
            return;
        }

        let vpn_status = match read_tunnel_status(&vpn_service).await {
            Ok(vpn_status) => vpn_status,
            Err(error) => {
                warn!("Failed to read the tunnel status: {error}");
                continue;
            }
        };
        if !is_current_status_stream(generation) {
            return;
        }

        if !vpn_status.connected {
            info!("VPN tunnel is no longer running; tearing it down.");
            if let Err(error) = disconnect_tunnel(&vpn_service).await {
                warn!("Failed to tear down the dropped tunnel: {error}");
            }
            publish_vpn_status(&app_handle, &vpn_status);
            return;
        }

        let Some(raw_metrics) = vpn_status.metrics.clone() else {
            continue;
        };
        let smoothed_metrics = rate_tracker.sample(raw_metrics);

        let is_persist_due =
            last_persisted_at.map_or(true, |persisted_at| {
                persisted_at.elapsed() >= METRICS_PERSIST_INTERVAL
            });
        if is_persist_due {
            last_persisted_at = Some(Instant::now());
            if let Some(ledger) = LedgerStore::open(&app_handle) {
                ledger.update_metrics(
                    &connected_instance.instance_id,
                    smoothed_metrics.bytes_sent,
                    smoothed_metrics.bytes_received,
                );
            }
        }

        publish_vpn_status(
            &app_handle,
            &VpnStatus {
                metrics: Some(smoothed_metrics),
                ..vpn_status
            },
        );
    }
}

async fn start_tunnel(
    vpn_service: &AndroidVpn<Wry>,
    tunnel_file_descriptor: RawFd,
    connect_params: VpnConnectParams,
) -> Result<ActiveTunnel> {
    let VpnConnectParams {
        instance_id,
        private_key,
        public_key,
        server_endpoint,
        region,
        provider,
        public_ip_v4,
        public_ip_v6,
        ..
    } = connect_params;

    let tun = unsafe { AsyncDevice::from_fd(tunnel_file_descriptor) }.map_err(|error| {
        ConfigurationError::TunnelConfiguration {
            reason: format!("Failed to open the VPN interface: {error}"),
        }
    })?;
    let udp = connect_protected_udp_socket(vpn_service, server_endpoint).await?;
    let wireguard_tunnel = create_wireguard_tunnel(private_key, public_key)?;

    let (shutdown_sender, shutdown_receiver) = watch::channel(());
    let mut tunnel = Tunnel::new(tun, udp, wireguard_tunnel, shutdown_receiver);
    let metrics = tunnel.metrics.clone();

    let task = tokio::spawn(async move {
        if let Err(error) = tunnel.run().await {
            error!("Tunnel exited: {error}");
        }
    });

    Ok(ActiveTunnel {
        shutdown: shutdown_sender,
        task,
        metrics,
        instance: ConnectedInstance {
            instance_id,
            public_ip_v4,
            public_ip_v6,
            region,
            provider,
        },
        connected_at: SystemTime::now(),
    })
}

async fn connect_protected_udp_socket(
    vpn_service: &AndroidVpn<Wry>,
    server_endpoint: SocketAddr,
) -> Result<UdpSocket> {
    let local_address = match server_endpoint {
        SocketAddr::V4(_) => SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
        SocketAddr::V6(_) => SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)),
    };

    let standard_socket =
        StandardUdpSocket::bind(local_address).map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("Failed to bind UDP socket: {error}"),
        })?;

    vpn_service
        .protect_socket(standard_socket.as_raw_fd())
        .await
        .map_err(convert_plugin_error)?;

    standard_socket
        .set_nonblocking(true)
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("Failed to make the UDP socket non-blocking: {error}"),
        })?;

    let socket =
        UdpSocket::from_std(standard_socket).map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("Failed to register the UDP socket: {error}"),
        })?;

    socket
        .connect(server_endpoint)
        .await
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("Failed to connect UDP socket to {server_endpoint}: {error}"),
        })?;

    info!("UDP socket connected to {}", server_endpoint);
    Ok(socket)
}

fn is_tunnel_running() -> Result<bool> {
    let mut active_tunnel = lock_active_tunnel()?;
    let is_running = active_tunnel
        .as_ref()
        .map_or(false, |tunnel| !tunnel.task.is_finished());
    if !is_running {
        *active_tunnel = None;
    }
    Ok(is_running)
}

fn read_active_tunnel_snapshot() -> Result<Option<ActiveTunnelSnapshot>> {
    Ok(lock_active_tunnel()?
        .as_ref()
        .map(|tunnel| ActiveTunnelSnapshot {
            is_finished: tunnel.task.is_finished(),
            instance: tunnel.instance.clone(),
            metrics: tunnel.metrics.clone(),
            connected_at: tunnel.connected_at,
        }))
}

fn lock_active_tunnel() -> Result<MutexGuard<'static, Option<ActiveTunnel>>> {
    ACTIVE_TUNNEL
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("ACTIVE_TUNNEL".to_string()).into())
}

fn is_current_status_stream(generation: u64) -> bool {
    STATUS_STREAM_GENERATION.load(Ordering::SeqCst) == generation
}

fn build_disconnected_status() -> VpnStatus {
    VpnStatus {
        connected: false,
        instance: None,
        metrics: None,
        connected_at: None,
        connection_error: None,
    }
}

fn serialize_response<T: Serialize>(response: &T) -> Result<Value> {
    serde_json::to_value(response).map_err(|error| {
        DaemonError::InvalidResponse {
            reason: error.to_string(),
        }
        .into()
    })
}

fn convert_plugin_error(error: PluginInvokeError) -> Error {
    DaemonError::CommandFailed {
        command: error.to_string(),
    }
    .into()
}
