use crate::types::DaemonCommand;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub const SOCKET_PATH: &str = "/tmp/myvpn.sock";

pub async fn send_command(cmd: DaemonCommand) -> anyhow::Result<String> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;
    let msg = serde_json::to_string(&cmd)? + "\n";
    stream.write_all(msg.as_bytes()).await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response)
}

pub fn is_daemon_running() -> bool {
    Path::new(SOCKET_PATH).exists()
}
