use std::{collections::VecDeque, net::SocketAddr};

use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use byocvpn_core::{
    error::{ConfigurationError, Result, SystemError},
    ipc::{IpcSocket, IpcStream},
    tunnel::{ConnectedInstance, Tunnel, TunnelMetrics},
};
use futures::StreamExt;
use ipnet::IpNet;
use log::*;
use net_route::Handle as RouteHandle;
use tokio::{net::UdpSocket, sync::watch};
use tun_rs::DeviceBuilder;

use crate::routing::dns::DomainNameSystemOverrideGuard;
use crate::{
    constants,
    routing::routes::{add_vpn_routes, update_server_host_route},
    tunnel_manager::{TUNNEL_MANAGER, TunnelHandle},
};

pub async fn connect_vpn(
    instance_id: String,
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    endpoint: SocketAddr,
    ipv4: IpNet,
    ipv6: IpNet,
    dns_servers: Vec<String>,
    region: String,
    provider: String,
    public_ip_v4: Option<String>,
    public_ip_v6: Option<String>,
) -> Result<()> {
    info!("Connecting VPN: instance={}, region={}, provider={}", instance_id, region, provider);

    let endpoint_ip = endpoint.ip().to_string();
    let (public_ip_v4, public_ip_v6) = match (public_ip_v4, public_ip_v6) {
        (Some(v4), v6) => (Some(v4), v6),
        (None, Some(v6)) => (None, Some(v6)),
        (None, None) => {
            if endpoint.is_ipv4() {
                (Some(endpoint_ip), None)
            } else {
                (None, Some(endpoint_ip))
            }
        }
    };

    #[cfg(target_os = "macos")]
    let tun_interface_name = "utun4";
    #[cfg(windows)]
    let tun_interface_name = "byocvpn";
    #[cfg(not(any(target_os = "macos", windows)))]
    let tun_interface_name = "tun0";

    let tun = DeviceBuilder::new()
        .name(tun_interface_name)
        .ipv4(ipv4.addr(), ipv4.prefix_len(), None)
        .ipv6(ipv6.addr(), ipv6.prefix_len())
        .mtu(1280)
        .build_async()
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("Failed to create TUN device: {}", error),
        })?;

    let iface_name = tun
        .name()
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("Failed to get TUN name: {}", error),
        })?;

    info!("Created TUN device: {}", iface_name);

    let is_tunnel_running = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?
        .as_ref()
        .map_or(false, |handle| !handle.task.is_finished());

    debug!("Previous Tunnel running: {}", is_tunnel_running);
    if is_tunnel_running {
        return Err(ConfigurationError::TunnelConfiguration {
            reason: "Tunnel already running".to_string(),
        }
        .into());
    }

    let private_key_bytes: [u8; 32] =
        private_key.as_slice().try_into().map_err(|_| {
            ConfigurationError::InvalidValue {
                field: "private_key".to_string(),
                reason: "Private key must be exactly 32 bytes".to_string(),
            }
        })?;
    let public_key_bytes: [u8; 32] = public_key.as_slice().try_into().map_err(|_| {
        ConfigurationError::InvalidValue {
            field: "public_key".to_string(),
            reason: "Public key must be exactly 32 bytes".to_string(),
        }
    })?;

    let tunn = Tunn::new(
        StaticSecret::from(private_key_bytes),
        PublicKey::from(public_key_bytes),
        None,
        Some(25),
        0,
        None,
    )
    .map_err(|error| ConfigurationError::TunnelConfiguration {
        reason: format!("Failed to create tunnel: {:?}", error),
    })?;

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
            reason: format!("failed to bind UDP socket: {}", error),
        })?;
    debug!(
        "{:?} UDP socket bound to {}",
        endpoint,
        udp.local_addr()
            .map_err(|error| SystemError::TunnelIoFailed {
                reason: format!("failed to get UDP local address: {}", error),
            })?
    );
    udp.connect(endpoint)
        .await
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("failed to connect UDP socket: {}", error),
        })?;
    info!("UDP socket connected to {}", endpoint);

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut tunnel = Tunnel::new(tun, udp, tunn, shutdown_rx);
    let metrics = tunnel.metrics.clone();

    let task = tokio::spawn(async move {
        if let Err(error) = tunnel.run().await {
            error!("Tunnel exited: {error}");
        }
    });

    let metrics_socket_path = constants::metrics_socket_path();
    let metrics_clone = metrics.clone();
    let (metrics_shutdown_tx, mut metrics_shutdown_rx) = watch::channel(());

    let metrics_task = tokio::spawn(async move {
        let mut listener = match IpcSocket::bind(metrics_socket_path.clone()).await {
            Ok(l) => {
                info!(
                    "[Metrics] Created metrics socket at {}",
                    metrics_socket_path.to_string_lossy()
                );
                l
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
                        let current_metrics = metrics_clone.read().await.clone();
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

                        if upload_history.len() > 10 {
                            upload_history.pop_front();
                        }
                        if download_history.len() > 10 {
                            download_history.pop_front();
                        }

                        let upload_rate = if !upload_history.is_empty() {
                            upload_history.iter().sum::<u64>() / upload_history.len() as u64
                        } else {
                            0
                        };

                        let download_rate = if !download_history.is_empty() {
                            download_history.iter().sum::<u64>() / download_history.len() as u64
                        } else {
                            0
                        };

                        let metrics = TunnelMetrics {
                            bytes_sent: current_metrics.bytes_sent,
                            bytes_received: current_metrics.bytes_received,
                            packets_sent: current_metrics.packets_sent,
                            packets_received: current_metrics.packets_received,
                            upload_rate,
                            download_rate,
                        };

                        if let Ok(json) = serde_json::to_string(&metrics) {
                            if stream.write_all(json.as_bytes()).await.is_err()
                                || stream.write_all(b"\n").await.is_err() {
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
    });

    info!("Tunnel task spawned");

    let mut manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| SystemError::MutexPoisoned("TUNNEL_MANAGER".to_string()))?;

    info!("Adding VPN routes...");
    add_vpn_routes(&iface_name, &endpoint.ip().to_string()).await?;

    let route_monitor_server_ip = endpoint.ip().to_string();
    let (route_monitor_shutdown_tx, mut route_monitor_shutdown_rx) = watch::channel(());
    let route_monitor_task = tokio::spawn(async move {
        let route_handle = match RouteHandle::new() {
            Ok(h) => h,
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
            .and_then(|r| r.gateway);
        let stream = route_handle.route_listen_stream();
        futures::pin_mut!(stream);
        loop {
            tokio::select! {
                Some(_event) = StreamExt::next(&mut stream) => {
                    update_server_host_route(&route_monitor_server_ip, &mut last_gateway).await;
                }
                _ = route_monitor_shutdown_rx.changed() => {
                    info!("[RouteMonitor] Stopping.");
                    break;
                }
            }
        }
    });

    info!("Configuring DNS...");
    let optional_domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard> = {
        let domain_name_system_servers = dns_servers;

        debug!(
            "Parsed DNS servers from config: {:?}",
            domain_name_system_servers
        );
        if domain_name_system_servers.is_empty() {
            warn!("No DNS servers found in [Interface] DNS; leaving system DNS unchanged.");
            None
        } else {
            let as_refs: Vec<&str> = domain_name_system_servers
                .iter()
                .map(|string| string.as_str())
                .collect();
            match DomainNameSystemOverrideGuard::apply_to_all_services(&as_refs) {
                Ok(guard) => Some(guard),
                Err(error) => {
                    warn!("Failed to apply DNS servers: {error}");
                    None
                }
            }
        }
    };

    *manager = Some(TunnelHandle {
        shutdown: shutdown_tx,
        task,
        metrics,
        metrics_task,
        metrics_shutdown: metrics_shutdown_tx,
        route_monitor_task,
        route_monitor_shutdown: route_monitor_shutdown_tx,
        domain_name_system_override_guard: optional_domain_name_system_override_guard,
        server_ip: endpoint.ip().to_string(),
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
