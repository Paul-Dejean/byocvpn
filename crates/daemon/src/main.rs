use byocvpn_core::error::Result;
use byocvpn_daemon::run_daemon;

#[tokio::main]
async fn main() -> Result<()> {
    run_daemon().await
}
