use std::net::SocketAddr;

use base64::{engine::general_purpose, Engine};
use boringtun::{
    noise::Tunn,
    x25519::{PublicKey, StaticSecret},
};
use byocvpn_core::{daemon_client::DaemonCommand, tunnel::Tunnel};
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

mod tunnel_manager;
use crate::tunnel_manager::{TunnelHandle, TUNNEL_MANAGER};

pub mod constants;
pub mod daemon_client;
use crate::dns_macos::DomainNameSystemOverrideGuard;

#[cfg(target_os = "macos")]
mod dns_macos;

pub async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = constants::socket_path().to_string_lossy().to_string();
    if fs::try_exists(&socket_path).await? {
        fs::remove_file(&socket_path).await?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Set socket permissions so non-root users can connect
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&socket_path)?.permissions();
        perms.set_mode(0o777); // Read/write for all users
        std::fs::set_permissions(&socket_path, perms)?;
    }

    println!("Daemon listening on {}", &socket_path);
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
                    disconnect_vpn().await;
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

async fn disconnect_vpn() {
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
            .expect("FATAL: TUNNEL_MANAGER mutex poisoned!");
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
            eprintln!("[VPN Disconnect] Warning: Failed to send shutdown signal (tunnel task likely already stopped).");
        }

        // Wait for the tunnel task to complete
        println!("[VPN Disconnect] Waiting for tunnel task to complete...");
        match handle.task.await {
            Ok(_) => println!("[VPN Disconnect] Tunnel task completed successfully."),
            Err(e) => eprintln!("[VPN Disconnect] Error: Tunnel task failed: {:?}", e),
        }
    } else {
        println!("[VPN Disconnect] No active tunnel found.");
    }

    println!("[VPN Disconnect] VPN disconnected. Daemon continues running.");
    // DON'T exit the process - daemon should keep running for new connections
}

async fn connect_vpn(config_path: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Daemon received connect: {}", &config_path);

    let config =
        Ini::load_from_file(&config_path).expect(&format!("Failed to read {}", &config_path));

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

    println!("{:?} Manager Tunnel running", manager.is_some());

    // let is_tunnel_running = !manager.as_ref().unwrap().task.is_finished();

    // println!("Tunnel running: {}", is_tunnel_running);

    // //Step 4: route internet traffic
    println!("Adding VPN routes...");
    add_vpn_routes(&iface_name, &endpoint.ip().to_string()).await;

    #[cfg(target_os = "macos")]
    let optional_domain_name_system_override_guard: Option<DomainNameSystemOverrideGuard> = {
        println!("getting dns");
        let domain_name_system_servers =
            parse_domain_name_system_servers_from_interface_section(interface);
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
        task: task,
        #[cfg(target_os = "macos")]
        domain_name_system_override_guard: optional_domain_name_system_override_guard,
    });

    println!("VPN setup complete.");
    Ok(())
}

async fn add_vpn_routes(iface_name: &str, server_ip: &str) {
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
        }
    }

    println!("Finished adding VPN routes");
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

    match handle.add(&route).await {
        Ok(_) => {
            println!("Added route: {} via {}", destination, interface);
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!(
                "Route already exists: {} via {} (skipping)",
                destination, interface
            );
        }
        Err(e) => {
            eprintln!(
                "Failed to add route {} via {}: {}",
                destination, interface, e
            );
            return Err(e.into());
        }
    }

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
    match handle.delete(&route).await {
        Ok(_) => {
            println!("Deleted route: {} dev {}", destination, interface);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Route not found: {} dev {} (already removed)",
                destination, interface
            );
        }
        Err(e) => {
            eprintln!(
                "Failed to delete route {} dev {}: {}",
                destination, interface, e
            );
            return Err(e.into());
        }
    }

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
