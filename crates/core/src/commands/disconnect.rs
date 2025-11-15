use crate::{
    daemon_client::{DaemonClient, DaemonCommand},
    error::Result,
};
pub async fn disconnect(daemon_client: &dyn DaemonClient) -> Result<()> {
    daemon_client
        .send_command(DaemonCommand::Disconnect)
        .await?;
    Ok(())
}
