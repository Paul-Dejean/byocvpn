use boringtun::noise::{Tunn, TunnResult};
use tokio::{
    net::UdpSocket,
    sync::watch,
    time::{Duration, Instant},
};
use tun_rs::AsyncDevice;

pub struct Tunnel {
    tun: AsyncDevice,
    udp: UdpSocket,
    wg: Tunn,
    shutdown_rx: watch::Receiver<()>,
}

impl Tunnel {
    pub fn new(
        tun: AsyncDevice,
        udp: UdpSocket,
        wg: Tunn,
        shutdown_rx: watch::Receiver<()>,
    ) -> Self {
        Tunnel {
            tun,
            udp,
            wg,
            shutdown_rx,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tun_buf = [0u8; 1500];
        let mut udp_buf = [0u8; 1500];
        let mut out_buf = [0u8; 1500];
        let mut last_keepalive = Instant::now();
        println!("[Tunnel] Starting tunnel...");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    println!("[Tunnel] Shutdown requested.");
                    break;
                }

                Ok(n) = self.tun.recv(&mut tun_buf) => {
                    // println!("[TUN] Read {} bytes", n);
                    match self.wg.encapsulate(&tun_buf[..n], &mut out_buf) {
                        TunnResult::WriteToNetwork(packet) => {
                            self.udp.send(packet).await?;
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(e) => {
                            eprintln!("encapsulate error: {:?}", e);
                        },
                        _ => {}
                    }
                }

                Ok((n, src)) = self.udp.recv_from(&mut udp_buf) => {
                    match self.wg.decapsulate(Some(src.ip()), &udp_buf[..n], &mut out_buf) {
                        TunnResult::WriteToTunnelV4(packet, _src_ip) => {

                            self.tun.send(packet).await?;
                        },
                        TunnResult::WriteToTunnelV6(packet, _src_ip) => {

                            self.tun.send(packet).await?;
                        },
                        TunnResult::WriteToNetwork(packet) => {

                            self.udp.send(packet).await?;
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(e) => {
                            eprintln!("decapsulate error: {:?}", e);
                        },
                    }
                }

                _ = tokio::time::sleep(Duration::from_secs(15)) => {
                    if last_keepalive.elapsed() >= Duration::from_secs(15) {
                        // Use empty packet to trigger a keepalive, if needed
                        match self.wg.encapsulate(&[], &mut out_buf) {
                            TunnResult::WriteToNetwork(packet) => {
                                self.udp.send(packet).await?;
                                last_keepalive = Instant::now();
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        println!("[Tunnel] Clean shutdown.");
        Ok(())
    }
}
