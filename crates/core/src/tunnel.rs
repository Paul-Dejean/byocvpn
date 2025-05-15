use boringtun::noise::{Tunn, TunnResult};

use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tun::AsyncDevice;

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

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut tun_buf = [0u8; 1500];
        let mut udp_buf = [0u8; 1500];
        let mut out_buf = [0u8; 1500];
        let mut last_keepalive = Instant::now();

        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    println!("[Tunnel] Shutdown requested.");
                    break;
                }

                Ok(n) = self.tun.read(&mut tun_buf) => {
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

                Ok(n) = self.udp.recv(&mut udp_buf) => {
                    match self.wg.decapsulate(None, &udp_buf[..n], &mut out_buf) {
                        TunnResult::WriteToTunnelV4(packet, _src_ip) => {
                            self.tun.write_all(packet).await?;
                        },
                        TunnResult::WriteToNetwork(reply) => {
                            self.udp.send(reply).await?;
                        },
                        TunnResult::Done => {},
                        TunnResult::Err(e) => {
                            eprintln!("decapsulate error: {:?}", e);
                        },
                        _ => {}
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
