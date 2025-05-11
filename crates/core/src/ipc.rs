use crate::types::DaemonCommand;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::Duration;
use tokio::time::sleep;

pub const SOCKET_PATH: &str = "/tmp/myvpn.sock";

pub async fn send_command(cmd: DaemonCommand) -> anyhow::Result<String> {
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
        if Path::new(path).exists() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Socket not found",
    ))
}

pub fn is_daemon_running() -> bool {
    Path::new(SOCKET_PATH).exists()
}
