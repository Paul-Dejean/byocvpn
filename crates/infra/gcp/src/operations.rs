use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    retry::{RetryPolicy, retry},
};
use google_cloud_compute_v1::model::Operation;
use google_cloud_lro::Poller;
use log::debug;
use serde::Deserialize;

use crate::{
    client::GcpClient,
    error::{convert_operation_error, convert_sdk_error},
};

pub async fn await_compute_operation<OperationPoller>(
    operation_name: &str,
    poller: OperationPoller,
) -> Result<()>
where
    OperationPoller: Poller<Operation, Operation>,
{
    let operation = poller
        .until_done()
        .await
        .map_err(|error| convert_sdk_error(operation_name, error))?;
    operation
        .to_result()
        .map(|_| ())
        .map_err(|error| convert_operation_error(operation_name, error))
}

pub async fn wait_for_service_usage_operation(
    client: &GcpClient,
    operation_url: &str,
    policy: RetryPolicy,
) -> Result<()> {
    retry(
        || async move {
            let operation: ServiceUsageOperation = client.get(operation_url).await?;
            if operation.done != Some(true) {
                debug!("[GCP] Operation pending: {operation_url}");
                return Err(NetworkProvisioningError::CloudOperationTimedOut {
                    operation: operation_url.to_string(),
                }
                .into());
            }
            match operation.error {
                Some(error) => Err(NetworkProvisioningError::CloudOperationFailed {
                    reason: error.message.unwrap_or_else(|| "unknown error".to_string()),
                }
                .into()),
                None => Ok(()),
            }
        },
        policy,
    )
    .await
}

#[derive(Deserialize)]
struct ServiceUsageOperation {
    done: Option<bool>,
    error: Option<ServiceUsageOperationError>,
}

#[derive(Deserialize)]
struct ServiceUsageOperationError {
    message: Option<String>,
}
