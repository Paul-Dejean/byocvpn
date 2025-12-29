use std::net::SocketAddr;

use base64::{Engine, engine::general_purpose};
use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use byocvpn_core::{
    daemon_client::DaemonCommand,
    error::{Error, Result},
    ipc::{IpcSocket, IpcStream},
    tunnel::{Tunnel, TunnelMetrics, TunnelMetricsWithRates, VpnStatus},
};
use ini::Ini;
use ipnet::IpNet;
use net_route::{Handle, Route};
use tokio::{net::UdpSocket, sync::watch};
use tun_rs::DeviceBuilder;

mod tunnel_manager;
use crate::tunnel_manager::{TUNNEL_MANAGER, TunnelHandle};

pub mod constants;
pub mod daemon_client;

use crate::dns_macos::DomainNameSystemOverrideGuard;

#[cfg(target_os = "macos")]
mod dns_macos;

pub async fn run_daemon() -> Result<()> {
    let socket_path = constants::socket_path();

    let listener = IpcSocket::bind(socket_path.clone()).await?;

    println!("Daemon listening on {}", socket_path.to_string_lossy());
    println!("process id: {}", std::process::id());

    loop {
        let mut stream = listener.accept().await?;

        while let Ok(Some(line)) = stream.read_message().await {
            println!("Daemon received: {line}");
            println!("process id: {}", std::process::id());
            match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(DaemonCommand::Connect { config_path }) => {
                    println!("Daemon received connect: {config_path}");
                    match connect_vpn(config_path).await {
                        Ok(_) => {
                            if stream.send_message("Connected!").await.is_err() {
                                eprintln!("Failed to send response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Connect error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message(&error_msg).await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::Disconnect) => match disconnect_vpn().await {
                    Ok(_) => {
                        if stream.send_message("Disconnected.").await.is_err() {
                            eprintln!("Failed to send response to client");
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Disconnect error: {}", e);
                        eprintln!("{}", error_msg);
                        if stream.send_message(&error_msg).await.is_err() {
                            eprintln!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Status) => match get_vpn_status().await {
                    Ok(status) => match serde_json::to_string(&status) {
                        Ok(json) => {
                            if stream.send_message(&json).await.is_err() {
                                eprintln!("Failed to send status response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Status serialization error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message(&error_msg).await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Status error: {}", e);
                        eprintln!("{}", error_msg);
                        if stream.send_message(&error_msg).await.is_err() {
                            eprintln!("Failed to send error response to client");
                        }
                    }
                },
                Ok(DaemonCommand::Stats) => {
                    let stats = get_current_metrics().await;
                    match serde_json::to_string(&stats) {
                        Ok(response) => {
                            if stream.send_message(&response).await.is_err() {
                                eprintln!("Failed to send stats response to client");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Stats serialization error: {}", e);
                            eprintln!("{}", error_msg);
                            if stream.send_message("null").await.is_err() {
                                eprintln!("Failed to send error response to client");
                            }
                        }
                    }
                }
                Ok(DaemonCommand::HealthCheck) => {
                    if stream.send_message("healthy").await.is_err() {
                        eprintln!("Failed to send health response to client");
                    }
                }

                Err(e) => {
                    let error_msg = format!("Invalid command: {}", e);
                    eprintln!("{}", error_msg);
                    if stream.send_message(&error_msg).await.is_err() {
                        eprintln!("Failed to send error response to client");
                    }
                }
            }
        }
    }
}

async fn disconnect_vpn() -> Result<()> {
    println!("[VPN Disconnect] Disconnecting VPN tunnel...");

    // Check if config file exists to get endpoint info for route cleanup
    if let Ok(config) = Ini::load_from_file("wg0.conf") {
        if let Some(peer) = config.section(Some("Peer")) {
            if let Some(endpoint_str) = peer.get("Endpoint") {
                if let Ok(endpoint) = endpoint_str.parse::<SocketAddr>() {
                    remove_vpn_routes("utun4", &endpoint.ip().to_string()).await;
                    println!("[VPN Disconnect] Removed VPN routes.");
                }
            }
        }
    }

    // Shut down the tunnel task
    let maybe_handle = {
        let mut manager_guard = TUNNEL_MANAGER
            .lock()
            .map_err(|_| Error::TunnelCreationError("Mutex poisoned".to_string()))?;
        manager_guard.take()
    };

    if let Some(mut handle) = maybe_handle {
        println!("[VPN Disconnect] Stopping tunnel task...");

        #[cfg(target_os = "macos")]
        if let Some(mut domain_name_system_override_guard) =
            handle.domain_name_system_override_guard.take()
        {
            if let Err(error) = domain_name_system_override_guard.restore_now() {
                eprintln!("[VPN Disconnect] Warning: Failed to restore DNS: {error}");
            } else {
                println!("[VPN Disconnect] Restored original DNS.");
            }
        }

        // Send the shutdown signal via the watch channel
        if handle.shutdown.send(()).is_ok() {
            println!("[VPN Disconnect] Shutdown signal sent to tunnel task.");
        } else {
            eprintln!(
                "[VPN Disconnect] Warning: Failed to send shutdown signal (tunnel task likely already stopped)."
            );
        }

        // Stop metrics broadcaster
        let _ = handle.metrics_shutdown.send(());
        println!("[VPN Disconnect] Metrics broadcaster stopped.");

        // Wait for the tunnel task to complete
        println!("[VPN Disconnect] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => println!("[VPN Disconnect] Tunnel task completed successfully."),
            Err(e) => eprintln!("[VPN Disconnect] Error: Tunnel task failed: {:?}", e),
        }

        // Wait for metrics task
        let _ = handle.metrics_task.await;
    } else {
        println!("[VPN Disconnect] No active tunnel found.");
    }

    println!("[VPN Disconnect] VPN disconnected. Daemon continues running.");
    Ok(())
}

async fn get_vpn_status() -> Result<VpnStatus> {
    let manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| Error::TunnelCreationError("Mutex poisoned".to_string()))?;

    if let Some(handle) = manager.as_ref() {
        let is_running = !handle.task.is_finished();

        Ok(VpnStatus {
            connected: is_running,
            instance_id: handle.instance_id.clone(),
            public_ip_v4: handle.public_ip_v4.clone(),
            public_ip_v6: handle.public_ip_v6.clone(),
        })
    } else {
        Ok(VpnStatus {
            connected: false,
            instance_id: None,
            public_ip_v4: None,
            public_ip_v6: None,
        })
    }
}

struct WireguardConfig {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    endpoint: SocketAddr,
    ipv4: IpNet,
    ipv6: IpNet,
    interface_section: ini::Properties,
}

async fn parse_wireguard_config(config_path: &str) -> Result<WireguardConfig> {
    // Parse config file
    let config = Ini::load_from_file(config_path)
        .map_err(|e| Error::ConfigParseError(format!("Failed to read config file: {}", e)))?;

    let interface = config
        .section(Some("Interface"))
        .ok_or_else(|| Error::InvalidConfig("[Interface] section missing".to_string()))?;
    let peer = config
        .section(Some("Peer"))
        .ok_or_else(|| Error::InvalidConfig("[Peer] section missing".to_string()))?;

    // Parse private key
    let private_key_str = interface
        .get("PrivateKey")
        .ok_or_else(|| Error::InvalidConfig("PrivateKey missing".to_string()))?;
    let private_key = general_purpose::STANDARD
        .decode(private_key_str)
        .map_err(|e| Error::InvalidConfig(format!("Invalid PrivateKey: {}", e)))?;

    // Parse addresses
    let addresses_str = interface
        .get("Address")
        .ok_or_else(|| Error::InvalidConfig("Address missing".to_string()))?;
    let addresses: Result<Vec<IpNet>> = addresses_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<IpNet>()
                .map_err(|e| Error::InvalidConfig(format!("Invalid address: {}", e)))
        })
        .collect();
    let addresses = addresses?;

    // Parse public key
    let public_key_str = peer
        .get("PublicKey")
        .ok_or_else(|| Error::InvalidConfig("PublicKey missing".to_string()))?;
    let public_key = general_purpose::STANDARD
        .decode(public_key_str)
        .map_err(|e| Error::InvalidConfig(format!("Invalid PublicKey: {}", e)))?;

    // Parse endpoint
    let endpoint_str = peer
        .get("Endpoint")
        .ok_or_else(|| Error::InvalidConfig("Endpoint missing".to_string()))?;
    let endpoint: SocketAddr = endpoint_str
        .parse()
        .map_err(|e| Error::InvalidConfig(format!("Invalid Endpoint: {}", e)))?;

    let ipv4 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv4())
        .ok_or_else(|| Error::InvalidConfig("No IPv4 address found".to_string()))?
        .clone();
    let ipv6 = addresses
        .iter()
        .find(|ip| ip.addr().is_ipv6())
        .ok_or_else(|| Error::InvalidConfig("No IPv6 address found".to_string()))?
        .clone();

    Ok(WireguardConfig {
        private_key,
        public_key,
        endpoint,
        ipv4,
        ipv6,
        interface_section: interface.clone(),
    })
}

async fn connect_vpn(config_path: String) -> Result<()> {
    println!("Daemon received connect: {}", &config_path);

    // Parse config file
    let wg_config = parse_wireguard_config(&config_path).await?;

    // Extract instance_id from config filename
    // Expected format: /path/to/{instance_id}.conf
    let instance_id = std::path::Path::new(&config_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // Extract IP addresses from endpoint
    let endpoint_ip = wg_config.endpoint.ip().to_string();
    let (public_ip_v4, public_ip_v6) = if wg_config.endpoint.is_ipv4() {
        (Some(endpoint_ip), None)
    } else {
        (None, Some(endpoint_ip))
    };

    // Create TUN device
    let tun = DeviceBuilder::new()
        .name("utun4")
        .ipv4(wg_config.ipv4.addr(), wg_config.ipv4.prefix_len(), None)
        .ipv6(wg_config.ipv6.addr(), wg_config.ipv6.prefix_len())
        .mtu(1280)
        .build_async()
        .map_err(|e| Error::TunnelCreationError(format!("Failed to create TUN device: {}", e)))?;

    let iface_name = tun
        .name()
        .map_err(|e| Error::TunnelCreationError(format!("Failed to get TUN name: {}", e)))?
        .to_string();
    println!("Created TUN device: {}", iface_name);

    // Check if tunnel is already running
    let is_tunnel_running = TUNNEL_MANAGER
        .lock()
        .map_err(|_| Error::TunnelCreationError("Mutex poisoned".to_string()))?
        .as_ref()
        .map_or(false, |handle| !handle.task.is_finished());

    println!("Previous Tunnel running: {}", is_tunnel_running);
    if is_tunnel_running {
        return Err(Error::TunnelCreationError(
            "Tunnel already running".to_string(),
        ));
    }

    println!("Creating Tunnel");

    // Convert keys
    let private_key_bytes: [u8; 32] =
        wg_config.private_key.as_slice().try_into().map_err(|_| {
            Error::InvalidConfig("Private key must be exactly 32 bytes".to_string())
        })?;
    let public_key_bytes: [u8; 32] = wg_config
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| Error::InvalidConfig("Public key must be exactly 32 bytes".to_string()))?;

    let tunn = Tunn::new(
        StaticSecret::from(private_key_bytes),
        PublicKey::from(public_key_bytes),
        None,     // preshared key
        Some(25), // keepalive
        0,
        None,
    )
    .map_err(|e| Error::TunnelCreationError(format!("Failed to create tunnel: {:?}", e)))?;

    println!("Created Tunn device");

    // Create UDP socket
    let local: SocketAddr = "0.0.0.0:0"
        .parse()
        .map_err(|e| Error::NetworkConfigError(format!("Invalid local address: {}", e)))?;
    let udp = UdpSocket::bind(local).await?;
    println!(
        "{:?} UDP socket bound to {}",
        wg_config.endpoint,
        udp.local_addr()?
    );
    udp.connect(wg_config.endpoint).await?;
    println!("UDP socket connected to {}", wg_config.endpoint);

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut tunnel = Tunnel::new(tun, udp, tunn, shutdown_rx);
    let metrics = tunnel.metrics.clone();

    let task = tokio::spawn(async move {
        if let Err(e) = tunnel.run().await {
            eprintln!("Tunnel exited: {e}");
        }
    });

    // Create metrics streaming task
    let metrics_socket_path = constants::metrics_socket_path();
    let metrics_clone = metrics.clone();
    let (metrics_shutdown_tx, mut metrics_shutdown_rx) = watch::channel(());

    let metrics_task = tokio::spawn(async move {
        let listener = match IpcSocket::bind(metrics_socket_path.clone()).await {
            Ok(l) => {
                println!(
                    "[Metrics] Created metrics socket at {}",
                    metrics_socket_path.to_string_lossy()
                );
                l
            }
            Err(e) => {
                eprintln!("[Metrics] Failed to create metrics socket: {}", e);
                return;
            }
        };

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        let mut last_metrics = TunnelMetrics::default();
        let mut last_time = tokio::time::Instant::now();
        let mut connected_stream: Option<IpcStream> = None;

        // Moving average buffers (10 samples = 10 seconds)
        use std::collections::VecDeque;
        let mut upload_history: VecDeque<u64> = VecDeque::with_capacity(10);
        let mut download_history: VecDeque<u64> = VecDeque::with_capacity(10);

        loop {
            tokio::select! {
                // Accept new connections (only keep one active)
                Ok(stream) = listener.accept() => {
                    println!("[Metrics] Client connected to metrics stream");
                    connected_stream = Some(stream);
                }

                _ = interval.tick() => {
                    if let Some(stream) = connected_stream.as_mut() {
                        let current_metrics = metrics_clone.read().await.clone();
                        let now = tokio::time::Instant::now();
                        let elapsed = now.duration_since(last_time).as_secs_f64();

                        // Calculate instantaneous rates (bytes per second)
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

                        // Add to history
                        upload_history.push_back(upload_rate_instant);
                        download_history.push_back(download_rate_instant);

                        // Keep only last 10 samples
                        if upload_history.len() > 10 {
                            upload_history.pop_front();
                        }
                        if download_history.len() > 10 {
                            download_history.pop_front();
                        }

                        // Calculate moving average
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

                        let metrics_with_rates = TunnelMetricsWithRates {
                            bytes_sent: current_metrics.bytes_sent,
                            bytes_received: current_metrics.bytes_received,
                            packets_sent: current_metrics.packets_sent,
                            packets_received: current_metrics.packets_received,
                            upload_rate,
                            download_rate,
                        };

                        // Write metrics as JSON to the stream
                        if let Ok(json) = serde_json::to_string(&metrics_with_rates) {
                            if stream.write_all(json.as_bytes()).await.is_err()
                                || stream.write_all(b"\n").await.is_err() {
                                println!("[Metrics] Client disconnected");
                                connected_stream = None;
                            }
                        }

                        last_metrics = current_metrics;
                        last_time = now;
                    }
                }

                _ = metrics_shutdown_rx.changed() => {
                    println!("[Metrics] Stopping metrics streamer");
                    break;
                }
            }
        }
    });

    println!("Tunnel task spawned");

    let mut manager = TUNNEL_MANAGER
        .lock()
        .map_err(|_| Error::TunnelCreationError("Mutex poisoned".to_string()))?;

    // Add VPN routes
    println!("Adding VPN routes...");
    add_vpn_routes(&iface_name, &wg_config.endpoint.ip().to_string()).await?;

    println!("Configuring DNS...");
    #[cfg(target_os = "macos")]
    let optional_domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard> = {
        let domain_name_system_servers =
            parse_domain_name_system_servers_from_interface_section(&wg_config.interface_section);
        println!(
            "Parsed DNS servers from config: {:?}",
            domain_name_system_servers
        );
        if domain_name_system_servers.is_empty() {
            eprintln!(
                "No DNS servers found in [Interface] DNS = ...; leaving system DNS unchanged."
            );
            None
        } else {
            let as_refs: Vec<&str> = domain_name_system_servers
                .iter()
                .map(|s| s.as_str())
                .collect();
            match DomainNameSystemOverrideGuard::apply_to_all_services(&as_refs) {
                Ok(guard) => Some(guard),
                Err(error) => {
                    eprintln!("Failed to apply DNS servers to macOS services: {error}");
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
        #[cfg(target_os = "macos")]
        domain_name_system_override_guard: optional_domain_name_system_override_guard,
        instance_id,
        public_ip_v4,
        public_ip_v6,
    });

    println!("VPN setup complete.");
    Ok(())
}

async fn add_vpn_routes(iface_name: &str, server_ip: &str) -> Result<()> {
    println!(
        "Adding VPN routes for server {} via interface {}",
        server_ip, iface_name
    );

    let server_route = format!("{}/32", server_ip);
    let routes = [
        (server_route.as_str(), "default"),
        ("0.0.0.0/1", iface_name),
        ("128.0.0.0/1", iface_name),
        ("::/1", iface_name),
        ("8000::/1", iface_name),
    ];

    for (destination, interface) in routes.iter() {
        if let Err(e) = add_route(destination, interface).await {
            eprintln!(
                "Warning: Failed to add route {} via {}: {}",
                destination, interface, e
            );
            // Continue with other routes even if one fails
        }
    }

    println!("Finished adding VPN routes");
    Ok(())
}

async fn remove_vpn_routes(iface_name: &str, server_ip: &str) {
    println!(
        "Removing VPN routes for server {} via interface {}",
        server_ip, iface_name
    );

    let server_route = format!("{}/32", server_ip);
    let routes = [
        (server_route.as_str(), "default"),
        ("0.0.0.0/1", iface_name),
        ("128.0.0.0/1", iface_name),
        ("::/1", iface_name),
        ("8000::/1", iface_name),
    ];

    for (destination, interface) in routes.iter() {
        if let Err(e) = delete_route(destination, interface).await {
            eprintln!(
                "Warning: Failed to remove route {} via {}: {}",
                destination, interface, e
            );
        }
    }

    println!("Finished removing VPN routes");
}

async fn add_route(destination: &str, interface: &str) -> Result<()> {
    println!("Adding route: {} via {}", destination, interface);

    let subnet: IpNet = destination
        .parse()
        .map_err(|e| Error::RouteError(format!("Invalid subnet {}: {}", destination, e)))?;

    let handle = Handle::new()?;
    let ifindex = get_ifindex(interface).await?;

    println!("Interface index: {}", ifindex);

    // Build the route
    let route = if interface == "default" {
        // Set the default route
        let default_route = handle
            .default_route()
            .await?
            .ok_or_else(|| Error::RouteError("No default route found".to_string()))?;
        let gateway = default_route
            .gateway
            .ok_or_else(|| Error::RouteError("Default route has no gateway".to_string()))?;
        Route::new(subnet.addr(), subnet.prefix_len()).with_gateway(gateway)
    } else {
        Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex)
    };

    println!("Route configuration: {:?}", route);

    match handle.add(&route).await {
        Ok(_) => {
            println!("Added route: {} via {}", destination, interface);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!(
                "Route already exists: {} via {} (skipping)",
                destination, interface
            );
            Ok(())
        }
        Err(e) => {
            let err_msg = format!(
                "Failed to add route {} via {}: {}",
                destination, interface, e
            );
            eprintln!("{}", err_msg);
            Err(Error::RouteError(err_msg))
        }
    }
}

async fn delete_route(destination: &str, interface: &str) -> Result<()> {
    let subnet: IpNet = destination
        .parse()
        .map_err(|e| Error::RouteError(format!("Invalid subnet {}: {}", destination, e)))?;

    let ifindex = get_ifindex(interface).await?;
    let handle = Handle::new()?;

    // Build the route
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    // Delete the route
    match handle.delete(&route).await {
        Ok(_) => {
            println!("Deleted route: {} dev {}", destination, interface);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Route not found: {} dev {} (already removed)",
                destination, interface
            );
            Ok(())
        }
        Err(e) => {
            let err_msg = format!(
                "Failed to delete route {} dev {}: {}",
                destination, interface, e
            );
            eprintln!("{}", err_msg);
            Err(Error::RouteError(err_msg))
        }
    }
}

/// Get current VPN metrics if tunnel is active
pub async fn get_current_metrics() -> Option<TunnelMetrics> {
    let manager = TUNNEL_MANAGER.lock().ok()?;
    if let Some(handle) = manager.as_ref() {
        let metrics = handle.metrics.read().await;
        Some(metrics.clone())
    } else {
        None
    }
}

/// Get the path to the metrics Unix socket
pub fn metrics_socket_path() -> std::path::PathBuf {
    constants::metrics_socket_path()
}

async fn get_ifindex(interface: &str) -> Result<u32> {
    let handle = Handle::new()?;

    if interface == "default" {
        // Get the default route
        let default_route = handle
            .default_route()
            .await?
            .ok_or_else(|| Error::NetworkConfigError("No default route found".to_string()))?;
        default_route.ifindex.ok_or_else(|| {
            Error::NetworkConfigError("Default route has no interface index".to_string())
        })
    } else {
        net_route::ifname_to_index(interface).ok_or_else(|| {
            Error::NetworkConfigError(format!("Failed to get interface index for {}", interface))
        })
    }
}

#[cfg(target_os = "macos")]
fn parse_domain_name_system_servers_from_interface_section(
    interface_section: &ini::Properties,
) -> Vec<String> {
    if let Some(value) = interface_section.get("DNS") {
        value
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    }
}
