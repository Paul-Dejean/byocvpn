use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};

use byocvpn_core::{
    daemon_client::VpnConnectParams,
    error::{Result, TunnelError},
    ipc::{IpcSocket, IpcStream, metrics_socket_path},
    tunnel::{
        ConnectedInstance, TUNNEL_MTU, Tunnel, TunnelMetrics, TunnelRateTracker,
        create_wireguard_tunnel,
    },
};
use futures::StreamExt;
use ipnet::IpNet;
use log::{debug, error, info, warn};
use net_route::Handle as RouteHandle;
use tokio::{
    net::UdpSocket,
    sync::{RwLock, watch},
    task::JoinHandle,
    time::{Duration, interval},
};
use tun_rs::{AsyncDevice, DeviceBuilder};

use crate::{
    firewall,
    routing::{
        dns::DnsOverrideGuard,
        routes::{add_vpn_routes, update_server_host_route},
    },
    tunnel_manager::{TunnelHandle, lock_tunnel_manager},
    vpn::session::{self, PersistedSession},
};

struct RoutingSetup {
    route_monitor_task: JoinHandle<()>,
    route_monitor_shutdown: watch::Sender<()>,
    dns_override_guard: Option<DnsOverrideGuard>,
}

struct RunningTunnel {
    shutdown: watch::Sender<()>,
    task: JoinHandle<()>,
    metrics: Arc<RwLock<TunnelMetrics>>,
    metrics_task: JoinHandle<()>,
    metrics_shutdown: watch::Sender<()>,
}

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
        kill_switch_enabled,
    } = params;

    info!("Connecting VPN: instance={instance_id}, region={region}, provider={provider}");
    ensure_no_tunnel_running()?;

    let (tun, interface_index) = setup_tun_device(private_ipv4, private_ipv6)?;
    let interface_name = tun
        .name()
        .unwrap_or_else(|_| format!("tun{interface_index}"));
    let server_ip = server_endpoint.ip().to_string();
    apply_kill_switch(kill_switch_enabled, &server_ip, &interface_name);

    let wireguard_tunnel = create_wireguard_tunnel(&private_key, &public_key)?;
    let udp = connect_udp_socket(server_endpoint).await?;
    let (shutdown_sender, shutdown_receiver) = watch::channel(());
    let running_tunnel = spawn_tunnel(
        Tunnel::new(tun, udp, wireguard_tunnel, shutdown_receiver),
        shutdown_sender,
    );

    let routing = setup_routing(interface_index, &server_ip, &dns_servers).await?;

    let connected_instance = ConnectedInstance {
        instance_id,
        public_ip_v4,
        public_ip_v6,
        region,
        provider,
    };
    persist_session(&connected_instance, kill_switch_enabled);
    register_tunnel_handle(TunnelHandle {
        shutdown: running_tunnel.shutdown,
        task: running_tunnel.task,
        metrics: running_tunnel.metrics,
        metrics_task: running_tunnel.metrics_task,
        metrics_shutdown: running_tunnel.metrics_shutdown,
        route_monitor_task: routing.route_monitor_task,
        route_monitor_shutdown: routing.route_monitor_shutdown,
        dns_override_guard: routing.dns_override_guard,
        server_ip,
        interface_index,
        instance: Some(connected_instance),
        connected_at: SystemTime::now(),
    })?;

    info!("VPN setup complete.");
    Ok(())
}

fn ensure_no_tunnel_running() -> Result<()> {
    let tunnel_already_running = lock_tunnel_manager()?
        .as_ref()
        .is_some_and(|handle| !handle.task.is_finished());
    if tunnel_already_running {
        return Err(TunnelError::AlreadyRunning.into());
    }
    Ok(())
}

fn apply_kill_switch(enabled: bool, server_ip: &str, interface_name: &str) {
    if !enabled {
        warn!("Session kill switch disabled; traffic can leak if the tunnel drops.");
        return;
    }
    if let Err(error) = firewall::apply(server_ip, interface_name) {
        warn!("Kill switch apply failed: {error}");
    }
}

fn spawn_tunnel(mut tunnel: Tunnel, shutdown: watch::Sender<()>) -> RunningTunnel {
    let metrics = tunnel.get_metrics();
    let task = tokio::spawn(async move {
        if let Err(error) = tunnel.run().await {
            error!("Tunnel exited: {error}");
        }
    });
    let (metrics_shutdown, metrics_shutdown_receiver) = watch::channel(());
    let metrics_task = spawn_metrics_task(Arc::clone(&metrics), metrics_shutdown_receiver);
    RunningTunnel {
        shutdown,
        task,
        metrics,
        metrics_task,
        metrics_shutdown,
    }
}

async fn setup_routing(
    interface_index: u32,
    server_ip: &str,
    dns_servers: &[String],
) -> Result<RoutingSetup> {
    info!("Adding VPN routes...");
    add_vpn_routes(interface_index, server_ip).await?;
    let (route_monitor_shutdown, route_monitor_shutdown_receiver) = watch::channel(());
    let route_monitor_task =
        spawn_route_monitor_task(server_ip.to_string(), route_monitor_shutdown_receiver);
    Ok(RoutingSetup {
        route_monitor_task,
        route_monitor_shutdown,
        dns_override_guard: apply_dns_servers(dns_servers),
    })
}

fn persist_session(connected_instance: &ConnectedInstance, kill_switch_enabled: bool) {
    let persisted_session = PersistedSession {
        instance_id: connected_instance.instance_id.clone(),
        provider: connected_instance.provider,
        region: connected_instance.region.clone(),
        public_ip_v4: connected_instance.public_ip_v4.clone(),
        public_ip_v6: connected_instance.public_ip_v6.clone(),
        kill_switch_enabled,
    };
    if let Err(error) = session::write_session(&persisted_session) {
        warn!("Failed to persist session: {error}");
    }
}

fn register_tunnel_handle(handle: TunnelHandle) -> Result<()> {
    *lock_tunnel_manager()? = Some(handle);
    Ok(())
}

fn setup_tun_device(private_ipv4: IpNet, private_ipv6: IpNet) -> Result<(AsyncDevice, u32)> {
    let tun = DeviceBuilder::new()
        .ipv4(private_ipv4.addr(), private_ipv4.prefix_len(), None)
        .ipv6(private_ipv6.addr(), private_ipv6.prefix_len())
        .mtu(TUNNEL_MTU)
        .build_async()
        .map_err(|source| TunnelError::TunDevice {
            operation: "create",
            source,
        })?;

    let interface_index = tun.if_index().map_err(|source| TunnelError::TunDevice {
        operation: "read the interface index of",
        source,
    })?;

    let interface_name = tun.name().unwrap_or_else(|_| "unknown".to_string());
    info!("Created TUN device: {interface_name} (index: {interface_index})");
    Ok((tun, interface_index))
}

async fn connect_udp_socket(server_endpoint: SocketAddr) -> Result<UdpSocket> {
    let local = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));

    let udp = UdpSocket::bind(local)
        .await
        .map_err(TunnelError::io("bind the UDP socket"))?;

    debug!(
        "UDP socket bound to {}",
        udp.local_addr()
            .map_err(TunnelError::io("read the UDP local address"))?
    );

    udp.connect(server_endpoint)
        .await
        .map_err(TunnelError::io("connect the UDP socket"))?;

    info!("UDP socket connected to {server_endpoint}");
    Ok(udp)
}

fn spawn_metrics_task(
    metrics: Arc<RwLock<TunnelMetrics>>,
    mut metrics_shutdown_rx: watch::Receiver<()>,
) -> JoinHandle<()> {
    let metrics_socket_path = metrics_socket_path();

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
                error!("[Metrics] Failed to create metrics socket: {error}");
                return;
            }
        };

        let mut interval = interval(Duration::from_secs(1));
        let mut rate_tracker = TunnelRateTracker::new();
        let mut connected_stream: Option<IpcStream> = None;

        loop {
            tokio::select! {
                Ok(stream) = listener.accept() => {
                    debug!("[Metrics] Client connected to metrics stream");
                    connected_stream = Some(stream);
                }

                _ = interval.tick() => {
                    if let Some(stream) = connected_stream.as_mut() {
                        let current_metrics = *metrics.read().await;
                        let snapshot = rate_tracker.sample(current_metrics);

                        if let Ok(json) = serde_json::to_string(&snapshot)
                            && (stream.write_all(json.as_bytes()).await.is_err()
                                || stream.write_all(b"\n").await.is_err())
                            {
                                debug!("[Metrics] Client disconnected");
                                connected_stream = None;
                            }
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
                error!("[RouteMonitor] Failed to create route handle: {error}");
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

fn apply_dns_servers(dns_servers: &[String]) -> Option<DnsOverrideGuard> {
    debug!("DNS servers from config: {dns_servers:?}");

    if dns_servers.is_empty() {
        warn!("No DNS servers found in config; leaving system DNS unchanged.");
        return None;
    }

    let as_refs: Vec<&str> = dns_servers.iter().map(String::as_str).collect();
    match DnsOverrideGuard::override_system_dns(&as_refs) {
        Ok(guard) => {
            info!("DNS servers applied: {dns_servers:?}");
            Some(guard)
        }
        Err(error) => {
            warn!("Failed to apply DNS servers: {error}");
            None
        }
    }
}
