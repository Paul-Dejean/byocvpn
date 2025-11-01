use byocvpn_daemon::run_daemon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_daemon().await
}
