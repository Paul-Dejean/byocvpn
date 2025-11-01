use std::net::SocketAddr;

use base64::{Engine, engine::general_purpose};
use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use ini::Ini;
use ipnet::IpNet;
use net_route::{Handle, Route};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UdpSocket, UnixListener},
    sync::watch,
};
use tun_rs::DeviceBuilder;

use crate::{
    ipc::SOCKET_PATH,
    tunnel::Tunnel,
    tunnel_manager::{TUNNEL_MANAGER, TunnelHandle},
    types::DaemonCommand,
};

pub async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    if fs::try_exists(SOCKET_PATH).await? {
        fs::remove_file(SOCKET_PATH).await?;
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
                    match connect_vpn(config_path).await {
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
    let config = Ini::load_from_file("wg0.conf").expect("Failed to read wg0.conf");
    let peer = config.section(Some("Peer")).expect("[Peer] missing");
    let endpoint: SocketAddr = peer.get("Endpoint").unwrap().parse().unwrap();
    remove_vpn_routes("utun4", &endpoint.ip().to_string()).await;
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

async fn connect_vpn(config_path: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Daemon received connect: {}", config_path);

    let config = Ini::load_from_file(config_path).expect("Failed to read wg0.conf");

    let interface = config
        .section(Some("Interface"))
        .expect("[Interface] missing");
    let peer = config.section(Some("Peer")).expect("[Peer] missing");

    let private_key = general_purpose::STANDARD
        .decode(interface.get("PrivateKey").unwrap())
        .unwrap();
    let addresses: Vec<IpNet> = interface
        .get("Address")
        .unwrap()
        .split(',')
        .map(|s| s.trim().parse::<IpNet>().unwrap())
        .collect();

    let public_key = general_purpose::STANDARD
        .decode(peer.get("PublicKey").unwrap())
        .unwrap();
    let endpoint: SocketAddr = peer.get("Endpoint").unwrap().parse().unwrap();

    let ipv4 = addresses.iter().find(|ip| ip.addr().is_ipv4()).unwrap();
    let ipv6 = addresses.iter().find(|ip| ip.addr().is_ipv6()).unwrap();

    let tun = DeviceBuilder::new()
        .name("utun4")
        .ipv4(ipv4.addr(), ipv4.prefix_len(), None)
        .ipv6(ipv6.addr(), ipv6.prefix_len())
        .mtu(1280)
        .build_async()
        .unwrap();
    let iface_name = tun.name().expect("Failed to get TUN name");
    println!("Created TUN device: {}", iface_name);

    // Step 2: boringtun
    let is_tunnel_running = TUNNEL_MANAGER
        .lock()
        .unwrap()
        .as_ref()
        .map_or(false, |handle| !handle.task.is_finished());
    println!("Previous Tunnel running: {}", is_tunnel_running);
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
    println!("{endpoint:?} UDP socket bound to {}", udp.local_addr()?);
    udp.connect(endpoint).await?;
    println!("UDP socket bound to {}", udp.local_addr()?);

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut tunnel = Tunnel::new(tun, udp, tunn, shutdown_rx);
    let task = tokio::spawn(async move {
        if let Err(e) = tunnel.run().await {
            eprintln!("Tunnel exited: {e}");
        }
    });

    println!("Tunnel task spawned: {:?}", task.is_finished());

    println!("{:?} Tunnel task spawned", task.id());
    // Step 2: boringtun

    println!("Before Manager Tunnel running: {}", is_tunnel_running);

    let mut manager = TUNNEL_MANAGER.lock().unwrap();
    println!("{:?} Manager Tunnel running", manager.is_some());
    *manager = Some(TunnelHandle {
        shutdown: shutdown_tx,
        task: task,
    });

    println!("{:?} Manager Tunnel running", manager.is_some());

    let is_tunnel_running = !manager.as_ref().unwrap().task.is_finished();

    println!("Tunnel running: {}", is_tunnel_running);

    // //Step 4: route internet traffic
    println!("Adding VPN routes...");
    add_vpn_routes(&iface_name, &endpoint.ip().to_string()).await;

    println!("VPN setup complete.");
    Ok(())
}

async fn add_vpn_routes(iface_name: &str, server_ip: &str) {
    add_route(&format!("{server_ip}/32"), "default")
        .await
        .unwrap();
    add_route("0.0.0.0/1", iface_name).await.unwrap();
    add_route("128.0.0.0/1", iface_name).await.unwrap();
    add_route("::/1", iface_name).await.unwrap();
    add_route("8000::/1", iface_name).await.unwrap();
}

async fn remove_vpn_routes(iface_name: &str, server_ip: &str) {
    delete_route(&format!("{server_ip}/32"), "default")
        .await
        .unwrap();
    delete_route("0.0.0.0/1", iface_name).await.unwrap();
    delete_route("128.0.0.0/1", iface_name).await.unwrap();
    delete_route("::/1", iface_name).await.unwrap();
    delete_route("8000::/1", iface_name).await.unwrap();
}

async fn add_route(destination: &str, interface: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("destination: {}", destination);
    let subnet: IpNet = destination.parse().unwrap();

    let handle = Handle::new()?;
    let ifindex = get_ifindex(interface).await;

    // Get the interface index
    println!("ifindex: {}", ifindex);

    // Build the route
    let route;
    if interface == "default" {
        // Set the default route
        let default_route = handle.default_route().await.unwrap().unwrap();
        route = Route::new(subnet.addr(), subnet.prefix_len())
            .with_gateway(default_route.gateway.unwrap());
    } else {
        route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);
    }

    println!("new route {:?}", route);

    handle.add(&route).await.expect("error adding route");

    println!("Added route: {} via {} ", destination, interface);

    Ok(())
}

async fn delete_route(
    destination: &str,
    interface: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let subnet: IpNet = destination.parse().unwrap();
    // Get the interface index
    let ifindex = get_ifindex(interface).await;

    let handle = Handle::new()?;

    // Build the route
    let route = Route::new(subnet.addr(), subnet.prefix_len()).with_ifindex(ifindex);

    // Delete the route
    handle.delete(&route).await?;
    println!("Deleted route: {} dev {}", destination, interface);

    Ok(())
}

async fn get_ifindex(interface: &str) -> u32 {
    let handle = Handle::new().unwrap();
    if interface == "default" {
        // Get the default route
        let default_route = handle.default_route().await.unwrap().unwrap();
        return default_route.ifindex.unwrap();
    } else {
        return net_route::ifname_to_index(interface).expect("Failed to get interface index");
    }
}
