use crate::ipc::SOCKET_PATH;
use crate::types::DaemonCommand;
use boringtun::noise::Tunn;
use ini::Ini;

use base64::{Engine, engine::general_purpose};

use ipnet::IpNet;
use serde::de;
use std::fs;
use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::tunnel::Tunnel;
use crate::tunnel_manager::TUNNEL_MANAGER;
use crate::tunnel_manager::TunnelHandle;
use boringtun::x25519::PublicKey;
use boringtun::x25519::StaticSecret;
use net_route::{Handle, Route};
use std::net::IpAddr;
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tun::{AbstractDevice, Configuration};

pub async fn run_daemon() -> anyhow::Result<()> {
    if std::path::Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("Daemon listening on {}", SOCKET_PATH);
    println!("process id: {}", std::process::id());
    loop {
        let (stream, _) = listener.accept().await?;
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        if let Ok(Some(line)) = lines.next_line().await {
            println!("Daemon received: {line}");
            println!("process id: {}", std::process::id());
            match serde_json::from_str::<DaemonCommand>(&line) {
                Ok(DaemonCommand::Connect { config_path }) => {
                    println!("Daemon received connect: {config_path}");
                    match connect_vpn(&config_path).await {
                        Ok(_) => {
                            writer.write_all(b"Connected!\n").await?;
                        }
                        Err(e) => {
                            writer
                                .write_all(format!("Connect error: {e}\n").as_bytes())
                                .await?;
                        }
                    }
                }
                Ok(DaemonCommand::Disconnect) => {
                    destroy_daemon().await;
                    writer.write_all(b"Disconnected.\n").await?;
                }
                Ok(DaemonCommand::Status) => {
                    writer.write_all(b"Status: dummy running\n").await?;
                }
                Err(e) => {
                    writer
                        .write_all(format!("Invalid command: {e}\n").as_bytes())
                        .await?;
                }
            }
        }
    }
}

async fn destroy_daemon() {
    println!("[Shutdown] Initiating graceful shutdown sequence...");
    remove_vpn_routes("utun4").await;
    println!("[Shutdown] Removed VPN routes.");

    // 1. & 2. Shut down and await the tunnel task
    // Acquire lock and take the handle out in a smaller scope
    // to ensure the lock is released before awaiting the task.
    let maybe_handle = {
        // Use expect for clearer panic message if mutex is poisoned
        let mut manager_guard = TUNNEL_MANAGER
            .lock()
            .expect("FATAL: TUNNEL_MANAGER mutex poisoned!");
        // take() removes the value from the Option, leaving None behind
        manager_guard.take()
    };

    if let Some(handle) = maybe_handle {
        println!("[Shutdown] Attempting to shut down active tunnel...");

        // Send the shutdown signal via the watch channel
        if handle.shutdown.send(()).is_ok() {
            println!("[Shutdown] Shutdown signal sent successfully to tunnel task.");
        } else {
            // This usually means the tunnel task has already exited and dropped the receiver.
            eprintln!(
                "[Shutdown] Warning: Failed to send shutdown signal (tunnel task likely already stopped)."
            );
        }

        // Wait for the tunnel task to complete, regardless of signal send success.
        println!("[Shutdown] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => println!("[Shutdown] Tunnel task completed successfully."),
            // The JoinError occurs if the task panicked.
            Err(e) => eprintln!(
                "[Shutdown] Error: Tunnel task failed or panicked during shutdown: {:?}",
                e
            ),
        }
    } else {
        println!("[Shutdown] No active tunnel found in manager.");
    }

    // 3. Clean up Unix socket file
    println!(
        "[Shutdown] Attempting to remove Unix socket file: {}",
        SOCKET_PATH
    );
    match std::fs::remove_file(SOCKET_PATH) {
        Ok(_) => println!("[Shutdown] Unix socket file removed successfully."),
        // It's okay if the file wasn't found (maybe already cleaned up or never created)
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("[Shutdown] Unix socket file not found (already removed?).");
        }
        // Log other errors
        Err(e) => {
            eprintln!(
                "[Shutdown] Error: Failed to remove Unix socket file '{}': {}",
                SOCKET_PATH, e
            );
        }
    }

    println!("[Shutdown] Graceful shutdown sequence finished.");

    std::process::exit(0);
}

async fn connect_vpn(config_path: &str) -> anyhow::Result<()> {
    println!("Daemon received connect: {}", config_path);

    let config = Ini::load_from_file("wg0.conf").expect("Failed to read wg0.conf");

    let interface = config
        .section(Some("Interface"))
        .expect("[Interface] missing");
    let peer = config.section(Some("Peer")).expect("[Peer] missing");

    let private_key = general_purpose::STANDARD
        .decode(interface.get("PrivateKey").unwrap())
        .unwrap();
    let address: IpNet = interface.get("Address").unwrap().parse().unwrap();

    let public_key = general_purpose::STANDARD
        .decode(peer.get("PublicKey").unwrap())
        .unwrap();
    let endpoint: SocketAddr = peer.get("Endpoint").unwrap().parse().unwrap();
    let allowed_ips: Vec<IpNet> = peer
        .get("AllowedIPs")
        .unwrap()
        .split(',')
        .map(|ip| ip.trim().parse().unwrap())
        .collect();

    // Step 1: TUN
    let mut config = Configuration::default();
    config
        .tun_name("utun4")
        .address(address.addr())
        .netmask(address.netmask())
        .mtu(1420)
        .up();

    let tun = tun::create_as_async(&config).expect("Failed to create TUN device");
    let iface_name = tun.tun_name().expect("Failed to get TUN name");
    println!("Created TUN device: {}", iface_name);

    // Step 2: boringtun
    let is_tunnel_running = TUNNEL_MANAGER
        .lock()
        .unwrap()
        .as_ref()
        .map_or(false, |handle| !handle.task.is_finished());
    println!("Tunnel running: {}", is_tunnel_running);
    if is_tunnel_running {
        return Ok(());
    }
    println!("Creating Tunel");
    let tunn = Tunn::new(
        StaticSecret::from(<[u8; 32]>::try_from(private_key.as_slice())?),
        PublicKey::from(<[u8; 32]>::try_from(public_key.as_slice())?),
        None,     // preshared key
        Some(25), // Vec<IpNet>
        0,
        None,
    )
    .expect("error creating tunn");
    println!("Created Tunn device");

    // Step 3: poll loop
    // Step 3: UDP socket + Tunnel loop
    let local = "0.0.0.0:0".parse::<SocketAddr>()?;
    let udp = UdpSocket::bind(local).await?;
    udp.connect(endpoint).await?;
    println!("UDP socket bound to {}", udp.local_addr()?);

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut tunnel = Tunnel::new(tun, udp, tunn, endpoint, shutdown_rx);
    let task = tokio::spawn(async move {
        if let Err(e) = tunnel.run().await {
            eprintln!("Tunnel exited: {e}");
        }
    });

    println!("{:?} Tunnel task spawned", task.id());

    println!("Tunnel running: {}", is_tunnel_running);

    let mut manager = TUNNEL_MANAGER.lock().unwrap();
    *manager = Some(TunnelHandle {
        shutdown: shutdown_tx,
        task,
    });

    // //Step 4: route internet traffic
    add_vpn_routes(&iface_name).await;

    println!("VPN setup complete.");
    Ok(())
}

async fn add_vpn_routes(iface_name: &str) {
    // split default route for compatibility
    add_route("0.0.0.0/1", "10.0.0.1", iface_name).await;
    add_route("128.0.0.0/1", "10.0.0.1", iface_name).await;
}

async fn remove_vpn_routes(iface_name: &str) {
    delete_route("0.0.0.0/1", iface_name).await;
    delete_route("128.0.0.0/1", iface_name).await;
}

async fn add_route(destination: &str, gateway: &str, interface: &str) -> anyhow::Result<()> {
    let subnet: IpNet = destination.parse().unwrap();
    let gateway_ip: IpAddr = gateway.parse().unwrap();
    // Get the interface index
    let ifindex = net_route::ifname_to_index(interface).expect("Failed to get interface index");

    let handle = Handle::new()?;

    // Build the route
    let route = Route::new(subnet.addr(), subnet.prefix_len())
        .with_ifindex(ifindex)
        .with_gateway(gateway_ip);

    // Add the route
    handle.add(&route).await?;
    println!(
        "Added route: {} via {} dev {}",
        destination, gateway, interface
    );

    Ok(())
}

async fn delete_route(destination: &str, interface: &str) -> anyhow::Result<()> {
    let subnet: IpNet = destination.parse().unwrap();
    // Get the interface index
    let ifindex = net_route::ifname_to_index(interface).expect("Failed to get interface index");

    let handle = Handle::new()?;

    // Build the route
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    // Delete the route
    handle.delete(&route).await?;
    println!("Deleted route: {} dev {}", destination, interface);

    Ok(())
}
