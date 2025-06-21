use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind},
    net::UnixStream,
    time::{Duration, sleep},
};

use crate::types::DaemonCommand;

pub const SOCKET_PATH: &str = "/tmp/myvpn.sock";

pub async fn send_command(cmd: DaemonCommand) -> Result<String, Box<dyn std::error::Error>> {
    wait_for_socket(SOCKET_PATH, 50).await?;
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;
    println!("Connected to daemon at {}", SOCKET_PATH);
    let msg = serde_json::to_string(&cmd)? + "\n";
    stream.write_all(msg.as_bytes()).await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response)
}

async fn wait_for_socket(path: &str, max_retries: u32) -> std::io::Result<()> {
    for _ in 0..max_retries {
        match UnixStream::connect(path).await {
            Ok(_) => return Ok(()), // Daemon is up and accepting
            Err(e)
                if e.kind() == ErrorKind::ConnectionRefused || e.kind() == ErrorKind::NotFound =>
            {
                sleep(Duration::from_millis(100)).await;
            }
            Err(e) => return Err(e), // Any other error (e.g., permission denied)
        }
    }

    Err(Error::new(
        ErrorKind::TimedOut,
        "Timed out waiting for daemon socket",
    ))
}

pub async fn is_daemon_running() -> bool {
    match UnixStream::connect(SOCKET_PATH).await {
        Ok(_) => true,   // Connection succeeded — daemon is alive
        Err(_) => false, // Connection failed — probably not running
    }
}
