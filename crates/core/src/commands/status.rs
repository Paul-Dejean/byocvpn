use crate::{
    daemon_client::{DaemonClient, DaemonCommand},
    error::{ConfigurationError, Result},
    tunnel::VpnStatus,
};

pub async fn fetch_vpn_status(client: &impl DaemonClient) -> Result<VpnStatus> {
    if !client.is_daemon_running().await {
        return Ok(VpnStatus {
            connected: false,
            instance: None,
            metrics: None,
        });
    }

    let response = client.send_command(DaemonCommand::Status).await?;

    serde_json::from_value(response).map_err(|error| {
        ConfigurationError::ParseError {
            value: "daemon status response".to_string(),
            reason: error.to_string(),
        }
        .into()
    })
}
