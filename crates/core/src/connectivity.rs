use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

pub fn can_connect_ipv6() -> bool {
    // Google's IPv6 DNS server (UDP/53, but TCP test works fine)
    let addr: SocketAddr = "[2001:4860:4860::8888]:53".parse().unwrap();

    // Try to open a TCP connection with a 2-second timeout
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
}
