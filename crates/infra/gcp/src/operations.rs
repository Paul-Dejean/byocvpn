use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    retry::{RetryPolicy, retry},
};
use log::debug;
use tokio::time::Duration;

use crate::{client::GcpClient, models::Operation};

pub const COMPUTE_OPERATION_POLL_POLICY: RetryPolicy =
    RetryPolicy::fixed(60, Duration::from_secs(3));

pub async fn wait_for_compute_operation(client: &GcpClient, operation: &Operation) -> Result<()> {
    let operation_url = operation.self_link.as_deref().ok_or_else(|| {
        NetworkProvisioningError::CloudOperationFailed {
            reason: "operation response missing selfLink".to_string(),
        }
    })?;
    wait_for_operation(client, operation_url, COMPUTE_OPERATION_POLL_POLICY).await
}

pub async fn wait_for_operation(
    client: &GcpClient,
    operation_url: &str,
    policy: RetryPolicy,
) -> Result<()> {
    retry(
        || async move {
            let operation: Operation = client.get(operation_url).await?;
            if !is_operation_done(&operation) {
                debug!("[GCP] Operation pending: {operation_url}");
                return Err(NetworkProvisioningError::CloudOperationTimedOut {
                    operation: operation_url.to_string(),
                }
                .into());
            }
            match operation.error {
                Some(error) => Err(NetworkProvisioningError::CloudOperationFailed {
                    reason: error
                        .errors
                        .as_ref()
                        .and_then(|details| details.first())
                        .and_then(|detail| detail.message.clone())
                        .or(error.message)
                        .unwrap_or_else(|| "unknown error".to_string()),
                }
                .into()),
                None => Ok(()),
            }
        },
        policy,
    )
    .await
}

fn is_operation_done(operation: &Operation) -> bool {
    operation.done == Some(true) || operation.status.as_deref() == Some("DONE")
}
