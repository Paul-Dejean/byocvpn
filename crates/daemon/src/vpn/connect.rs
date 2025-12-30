use std::{net::SocketAddr, path::Path};

use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use byocvpn_core::{
    error::{Error, Result},
    ipc::{IpcSocket, IpcStream},
    tunnel::{Tunnel, TunnelMetrics, TunnelMetricsWithRates},
};
use tokio::{net::UdpSocket, sync::watch};
use tun_rs::DeviceBuilder;

use crate::{
    constants,
    routing::{dns_macos::DomainNameSystemOverrideGuard, routes::add_vpn_routes},
    tunnel_manager::{TUNNEL_MANAGER, TunnelHandle},
    vpn::config::parse_wireguard_config,
};

pub async fn connect_vpn(config_path: String) -> Result<()> {
    println!("Daemon received connect: {}", &config_path);

    // Parse config file
    let wg_config = parse_wireguard_config(&config_path).await?;

    // Extract instance_id from config filename
    // Expected format: /path/to/{instance_id}.conf
    let instance_id = Path::new(&config_path)
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
        let domain_name_system_servers = wg_config.dns_servers;

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
