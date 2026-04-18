use std::{collections::VecDeque, net::SocketAddr, sync::Arc};

use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use byocvpn_core::{
    daemon_client::VpnConnectParams,
    error::{ConfigurationError, Result, SystemError},
    ipc::{IpcSocket, IpcStream},
    tunnel::{ConnectedInstance, Tunnel, TunnelMetrics},
};
use futures::StreamExt;
use ipnet::IpNet;
use log::*;
use net_route::Handle as RouteHandle;
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    task::JoinHandle,
};
use tun_rs::{AsyncDevice, DeviceBuilder};

use crate::{
    constants,
    routing::{
        dns::DnsOverrideGuard,
        routes::{add_vpn_routes, update_server_host_route},
    },
    tunnel_manager::{TUNNEL_MANAGER, TunnelHandle},
};

pub async fn connect_vpn(params: VpnConnectParams) -> Result<()> {
    let VpnConnectParams {
        instance_id,
        private_key,
        public_key,
        server_endpoint,
        private_ipv4,
        private_ipv6,
        dns_servers,
        region,
        provider,
        public_ip_v4,
        public_ip_v6,
    } = params;

    info!(
        "Connecting VPN: instance={}, region={}, provider={}",
        instance_id, region, provider
    );

    let tunnel_already_running = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?
        .as_ref()
        .map_or(false, |handle| !handle.task.is_finished());

    if tunnel_already_running {
        error!("Connect requested but a tunnel is already running.");
        return Err(ConfigurationError::TunnelConfiguration {
            reason: "Tunnel already running".to_string(),
        }
        .into());
    }

    let (tun, created_interface_name) = setup_tun_device(private_ipv4, private_ipv6)?;
    let wireguard_tunnel = create_wireguard_tunnel(private_key, public_key)?;
    let udp = connect_udp_socket(server_endpoint).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let mut tunnel = Tunnel::new(tun, udp, wireguard_tunnel, shutdown_rx);
    let metrics = tunnel.metrics.clone();

    let task = tokio::spawn(async move {
        if let Err(error) = tunnel.run().await {
            error!("Tunnel exited: {error}");
        }
    });

    let (metrics_shutdown_tx, metrics_shutdown_rx) = watch::channel(());
    let metrics_task = spawn_metrics_task(metrics.clone(), metrics_shutdown_rx);

    let mut manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;

    info!("Adding VPN routes...");
    add_vpn_routes(&created_interface_name, &server_endpoint.ip().to_string()).await?;

    let (route_monitor_shutdown_tx, route_monitor_shutdown_rx) = watch::channel(());
    let route_monitor_task =
        spawn_route_monitor_task(server_endpoint.ip().to_string(), route_monitor_shutdown_rx);

    let dns_override_guard = apply_dns_servers(dns_servers);

    *manager = Some(TunnelHandle {
        shutdown: shutdown_tx,
        task,
        metrics,
        metrics_task,
        metrics_shutdown: metrics_shutdown_tx,
        route_monitor_task,
        route_monitor_shutdown: route_monitor_shutdown_tx,
        dns_override_guard,
        server_ip: server_endpoint.ip().to_string(),
        instance: Some(ConnectedInstance {
            instance_id,
            public_ip_v4,
            public_ip_v6,
            region,
            provider,
        }),
    });

    info!("VPN setup complete.");
    Ok(())
}

fn setup_tun_device(private_ipv4: IpNet, private_ipv6: IpNet) -> Result<(AsyncDevice, String)> {
    let tun = DeviceBuilder::new()
        .name(constants::get_interface_name())
        .ipv4(private_ipv4.addr(), private_ipv4.prefix_len(), None)
        .ipv6(private_ipv6.addr(), private_ipv6.prefix_len())
        .mtu(constants::TUNNEL_MTU)
        .build_async()
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("Failed to create TUN device: {}", error),
        })?;

    let created_interface_name =
        tun.name()
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("Failed to get TUN name: {}", error),
            })?;

    info!("Created TUN device: {}", created_interface_name);
    Ok((tun, created_interface_name))
}

fn create_wireguard_tunnel(private_key: Vec<u8>, public_key: Vec<u8>) -> Result<Tunn> {
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

async fn connect_udp_socket(server_endpoint: SocketAddr) -> Result<UdpSocket> {
    let local: SocketAddr =
        "0.0.0.0:0"
            .parse()
            .map_err(|error| ConfigurationError::ParseError {
                reason: format!("Failed to parse local socket address: {}", error),
                value: "0.0.0.0".to_string(),
            })?;

    let udp = UdpSocket::bind(local)
        .await
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("Failed to bind UDP socket: {}", error),
        })?;

    debug!(
        "UDP socket bound to {}",
        udp.local_addr()
            .map_err(|error| SystemError::TunnelIoFailed {
                reason: format!("Failed to get UDP local address: {}", error),
            })?
    );

    udp.connect(server_endpoint)
        .await
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!(
                "Failed to connect UDP socket to {}: {}",
                server_endpoint, error
            ),
        })?;

    info!("UDP socket connected to {}", server_endpoint);
    Ok(udp)
}

fn spawn_metrics_task(
    metrics: Arc<RwLock<TunnelMetrics>>,
    mut metrics_shutdown_rx: watch::Receiver<()>,
) -> JoinHandle<()> {
    let metrics_socket_path = constants::metrics_socket_path();

    tokio::spawn(async move {
        let mut listener = match IpcSocket::bind(metrics_socket_path.clone()).await {
            Ok(listener) => {
                info!(
                    "[Metrics] Created metrics socket at {}",
                    metrics_socket_path.to_string_lossy()
                );
                listener
            }
            Err(error) => {
                error!("[Metrics] Failed to create metrics socket: {}", error);
                return;
            }
        };

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        let mut last_metrics = TunnelMetrics::default();
        let mut last_time = tokio::time::Instant::now();
        let mut connected_stream: Option<IpcStream> = None;
        let mut upload_history: VecDeque<u64> = VecDeque::with_capacity(10);
        let mut download_history: VecDeque<u64> = VecDeque::with_capacity(10);

        loop {
            tokio::select! {
                Ok(stream) = listener.accept() => {
                    debug!("[Metrics] Client connected to metrics stream");
                    connected_stream = Some(stream);
                }

                _ = interval.tick() => {
                    if let Some(stream) = connected_stream.as_mut() {
                        let current_metrics = metrics.read().await.clone();
                        let now = tokio::time::Instant::now();
                        let elapsed = now.duration_since(last_time).as_secs_f64();

                        let upload_rate_instant = if elapsed > 0.0 {
                            ((current_metrics.bytes_sent - last_metrics.bytes_sent) as f64 / elapsed) as u64
                        } else {
                            0
                        };
                        let download_rate_instant = if elapsed > 0.0 {
                            ((current_metrics.bytes_received - last_metrics.bytes_received) as f64 / elapsed) as u64
                        } else {
                            0
                        };

                        upload_history.push_back(upload_rate_instant);
                        download_history.push_back(download_rate_instant);
                        if upload_history.len() > 10 { upload_history.pop_front(); }
                        if download_history.len() > 10 { download_history.pop_front(); }

                        let upload_rate = if !upload_history.is_empty() {
                            upload_history.iter().sum::<u64>() / upload_history.len() as u64
                        } else { 0 };
                        let download_rate = if !download_history.is_empty() {
                            download_history.iter().sum::<u64>() / download_history.len() as u64
                        } else { 0 };

                        let snapshot = TunnelMetrics {
                            bytes_sent: current_metrics.bytes_sent,
                            bytes_received: current_metrics.bytes_received,
                            packets_sent: current_metrics.packets_sent,
                            packets_received: current_metrics.packets_received,
                            upload_rate,
                            download_rate,
                        };

                        if let Ok(json) = serde_json::to_string(&snapshot) {
                            if stream.write_all(json.as_bytes()).await.is_err()
                                || stream.write_all(b"\n").await.is_err()
                            {
                                debug!("[Metrics] Client disconnected");
                                connected_stream = None;
                            }
                        }

                        last_metrics = current_metrics;
                        last_time = now;
                    }
                }

                _ = metrics_shutdown_rx.changed() => {
                    info!("[Metrics] Stopping metrics streamer");
                    break;
                }
            }
        }
    })
}

fn spawn_route_monitor_task(
    server_ip: String,
    mut route_monitor_shutdown_rx: watch::Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let route_handle = match RouteHandle::new() {
            Ok(handle) => handle,
            Err(error) => {
                error!("[RouteMonitor] Failed to create route handle: {}", error);
                return;
            }
        };

        let mut last_gateway = route_handle
            .default_route()
            .await
            .ok()
            .flatten()
            .and_then(|route| route.gateway);

        let stream = route_handle.route_listen_stream();
        futures::pin_mut!(stream);

        loop {
            tokio::select! {
                Some(_event) = StreamExt::next(&mut stream) => {
                    update_server_host_route(&server_ip, &mut last_gateway).await;
                }
                _ = route_monitor_shutdown_rx.changed() => {
                    info!("[RouteMonitor] Stopping.");
                    break;
                }
            }
        }
    })
}

fn apply_dns_servers(dns_servers: Vec<String>) -> Option<DnsOverrideGuard> {
    debug!("DNS servers from config: {:?}", dns_servers);

    if dns_servers.is_empty() {
        warn!("No DNS servers found in config; leaving system DNS unchanged.");
        return None;
    }

    let as_refs: Vec<&str> = dns_servers.iter().map(|s| s.as_str()).collect();
    match DnsOverrideGuard::override_system_dns(&as_refs) {
        Ok(guard) => {
            info!("DNS servers applied: {:?}", dns_servers);
            Some(guard)
        }
        Err(error) => {
            warn!("Failed to apply DNS servers: {error}");
            None
        }
    }
}
