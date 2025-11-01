use crate::daemon_client::{DaemonClient, DaemonCommand};
pub async fn disconnect(
    daemon_client: &dyn DaemonClient,
) -> Result<(), Box<dyn std::error::Error>> {
    daemon_client
        .send_command(DaemonCommand::Disconnect)
        .await?;
    Ok(())
}
