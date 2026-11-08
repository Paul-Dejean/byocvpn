use byocvpn_core::retry::{RetryPolicy, retry};
use google_cloud_compute_v1::model::Operation;
use google_cloud_lro::Poller;
use log::debug;
use serde::Deserialize;

use crate::{
    client::GcpClient,
    error::{GcpError, LabelOperation, Result},
};

pub async fn await_compute_operation<OperationPoller>(
    operation_name: &'static str,
    poller: OperationPoller,
) -> Result<()>
where
    OperationPoller: Poller<Operation, Operation>,
{
    let operation = poller.until_done().await.label_operation(operation_name)?;
    operation.to_result().label_operation(operation_name)?;
    Ok(())
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
                return Err(GcpError::ServiceUsageOperationTimedOut {
                    operation_url: operation_url.to_string(),
                });
            }
            match operation.error {
                Some(error) => Err(GcpError::ServiceUsageOperationFailed {
                    message: error.message.unwrap_or_else(|| "unknown error".to_string()),
                }),
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
