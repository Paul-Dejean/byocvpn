use std::{collections::VecDeque, net::SocketAddr, path::Path};

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
use log::*;
use net_route::Handle as RouteHandle;
use tokio::{net::UdpSocket, sync::watch};
use tun_rs::DeviceBuilder;

#[cfg(target_os = "linux")]
use crate::routing::dns_linux::DomainNameSystemOverrideGuard;
#[cfg(target_os = "macos")]
use crate::routing::dns_macos::DomainNameSystemOverrideGuard;
#[cfg(windows)]
use crate::routing::dns_windows::DomainNameSystemOverrideGuard;
use crate::{
    constants,
    routing::routes::{add_vpn_routes, update_server_host_route},
    tunnel_manager::{TUNNEL_MANAGER, TunnelHandle},
    vpn::config::parse_wireguard_config,
};

pub async fn connect_vpn(
    config_path: String,
    region: String,
    provider: String,
    public_ip_v4: Option<String>,
    public_ip_v6: Option<String>,
) -> Result<()> {
    info!("Daemon received connect: {}", &config_path);

    let wg_config = parse_wireguard_config(&config_path).await?;

    let instance_id = Path::new(&config_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or(ConfigurationError::InvalidValue {
            field: "filename".to_string(),
            reason: "unable to extract instance ID".to_string(),
        })?;

    let endpoint_ip = wg_config.endpoint.ip().to_string();
    let (public_ip_v4, public_ip_v6) = match (public_ip_v4, public_ip_v6) {
        (Some(v4), v6) => (Some(v4), v6),
        (None, Some(v6)) => (None, Some(v6)),
        (None, None) => {
            if wg_config.endpoint.is_ipv4() {
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
        .ipv4(wg_config.ipv4.addr(), wg_config.ipv4.prefix_len(), None)
        .ipv6(wg_config.ipv6.addr(), wg_config.ipv6.prefix_len())
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

    info!("Previous Tunnel running: {}", is_tunnel_running);
    if is_tunnel_running {
        return Err(ConfigurationError::TunnelConfiguration {
            reason: "Tunnel already running".to_string(),
        }
        .into());
    }

    info!("Creating Tunnel");

    let private_key_bytes: [u8; 32] =
        wg_config.private_key.as_slice().try_into().map_err(|_| {
            ConfigurationError::InvalidValue {
                field: "private_key".to_string(),
                reason: "Private key must be exactly 32 bytes".to_string(),
            }
        })?;
    let public_key_bytes: [u8; 32] = wg_config.public_key.as_slice().try_into().map_err(|_| {
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

    info!("Created Tunn device");

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
    info!(
        "{:?} UDP socket bound to {}",
        wg_config.endpoint,
        udp.local_addr()
            .map_err(|error| SystemError::TunnelIoFailed {
                reason: format!("failed to get UDP local address: {}", error),
            })?
    );
    udp.connect(wg_config.endpoint)
        .await
        .map_err(|error| SystemError::TunnelIoFailed {
            reason: format!("failed to connect UDP socket: {}", error),
        })?;
    info!("UDP socket connected to {}", wg_config.endpoint);

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
        let listener = match IpcSocket::bind(metrics_socket_path.clone()).await {
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
                    info!("[Metrics] Client connected to metrics stream");
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
                                info!("[Metrics] Client disconnected");
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
    add_vpn_routes(&iface_name, &wg_config.endpoint.ip().to_string()).await?;

    let route_monitor_server_ip = wg_config.endpoint.ip().to_string();
    let (route_monitor_shutdown_tx, mut route_monitor_shutdown_rx) = watch::channel(());
    let route_monitor_task = tokio::spawn(async move {
        let route_handle = match RouteHandle::new() {
            Ok(h) => h,
            Err(error) => {
                error!("[RouteMonitor] Failed to create route handle: {}", error);
                return;
            }
        };
        let stream = route_handle.route_listen_stream();
        futures::pin_mut!(stream);
        loop {
            tokio::select! {
                Some(_event) = StreamExt::next(&mut stream) => {
                    update_server_host_route(&route_monitor_server_ip).await;
                }
                _ = route_monitor_shutdown_rx.changed() => {
                    info!("[RouteMonitor] Stopping.");
                    break;
                }
            }
        }
    });

    info!("Configuring DNS...");
    #[cfg(any(target_os = "macos", target_os = "linux", windows))]
    let optional_domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard> = {
        let domain_name_system_servers = wg_config.dns_servers;

        info!(
            "Parsed DNS servers from config: {:?}",
            domain_name_system_servers
        );
        if domain_name_system_servers.is_empty() {
            error!("No DNS servers found in [Interface] DNS = ...; leaving system DNS unchanged.");
            None
        } else {
            let as_refs: Vec<&str> = domain_name_system_servers
                .iter()
                .map(|string| string.as_str())
                .collect();
            match DomainNameSystemOverrideGuard::apply_to_all_services(&as_refs) {
                Ok(guard) => Some(guard),
                Err(error) => {
                    error!("Failed to apply DNS servers: {error}");
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
        #[cfg(any(target_os = "macos", target_os = "linux", windows))]
        domain_name_system_override_guard: optional_domain_name_system_override_guard,
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
