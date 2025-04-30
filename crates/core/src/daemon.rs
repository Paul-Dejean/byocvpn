use crate::ipc::SOCKET_PATH;
use crate::types::DaemonCommand;
use boringtun::noise::Tunn;
use ini::Ini;

use base64::{Engine, engine::general_purpose};

use ipnet::IpNet;
use std::fs;
use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::tunnel::Tunnel;
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

    loop {
        let (stream, _) = listener.accept().await?;
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        if let Ok(Some(line)) = lines.next_line().await {
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
                    println!("Daemon received disconnect");
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
        .address(address.addr())
        .netmask(address.netmask())
        .mtu(1420)
        .up();

    let tun = tun::create_as_async(&config).expect("Failed to create TUN device");
    let iface_name = tun.tun_name().expect("Failed to get TUN name");
    println!("Created TUN device: {}", iface_name);

    // Step 2: boringtun

    let tunn = Tunn::new(
        StaticSecret::from(<[u8; 32]>::try_from(private_key.as_slice())?),
        PublicKey::from(<[u8; 32]>::try_from(public_key.as_slice())?),
        None,     // preshared key
        Some(25), // Vec<IpNet>
        0,
        None,
    )
    .expect("error creating tunn");

    // Step 3: poll loop
    // Step 3: UDP socket + Tunnel loop
    let local = "0.0.0.0:0".parse::<SocketAddr>()?;
    let udp = UdpSocket::bind(local).await?;
    udp.connect(endpoint).await?;

    let (_shutdown_tx, shutdown_rx) = watch::channel(());

    let mut tunnel = Tunnel::new(tun, udp, tunn, endpoint, shutdown_rx);
    tokio::spawn(async move {
        if let Err(e) = tunnel.run().await {
            eprintln!("Tunnel exited with error: {e}");
        }
    });

    // Step 4: route internet traffic
    for net in allowed_ips {
        if net == "0.0.0.0/0".parse::<IpNet>().unwrap() {
            // split default route for compatibility
            add_route("0.0.0.0/1", "10.0.0.1", &iface_name);
            add_route("128.0.0.0/1", "10.0.0.1", &iface_name);
        } else {
            add_route(&net.to_string(), "10.0.0.1", &iface_name);
        }
    }

    println!("VPN setup complete.");
    Ok(())
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
