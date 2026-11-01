use byocvpn_core::error::{Error, NetworkProvisioningError, Result};
use byocvpn_core::retry::{RetryPolicy, retry_if};
use log::{debug, error, info, warn};
use serde::{Serialize, de::DeserializeOwned};
use tokio::time::Duration;

use crate::arm_error::{ArmErrorCode, extract_arm_error_code};
use crate::arm_resource::ArmResource;
use crate::client::AzureClient;
use crate::provider_registration::request_network_provider_registration;

const PROVIDER_REGISTRATION_RETRY_POLICY: RetryPolicy =
    RetryPolicy::exponential(30, Duration::from_secs(5), Duration::from_secs(60));

pub async fn get_existing<T: DeserializeOwned>(
    client: &AzureClient,
    resource: &ArmResource,
) -> Result<Option<T>> {
    match client.get::<T>(&client.build_resource_url(resource)).await {
        Ok(existing) => Ok(Some(existing)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

pub async fn put_and_wait<B: Serialize>(
    client: &AzureClient,
    resource: &ArmResource,
    body: &B,
) -> Result<()> {
    let async_operation_url = put_with_provider_retry(client, resource, body).await?;
    if let Some(operation_url) = async_operation_url {
        debug!("[Azure] Waiting for {resource} to be provisioned...");
        client.wait_for_async_operation(&operation_url).await?;
    }
    Ok(())
}

pub async fn delete_resource(client: &AzureClient, resource: &ArmResource) {
    let url = client.build_resource_url(resource);
    info!("[Azure] Deleting {resource}...");
    match client.delete(&url).await {
        Ok(Some(operation_url)) => {
            if let Err(error) = client.wait_for_async_operation(&operation_url).await {
                error!("[Azure] Failed to wait for {resource} deletion: {error}");
            } else {
                info!("[Azure] {resource} deleted.");
            }
        }
        Ok(None) => info!("[Azure] {resource} deleted."),
        Err(error) => error!("[Azure] Failed to delete {resource}: {error}"),
    }
}

async fn put_with_provider_retry<B: Serialize>(
    client: &AzureClient,
    resource: &ArmResource,
    body: &B,
) -> Result<Option<String>> {
    let url = client.build_resource_url(resource);
    let url = url.as_str();
    retry_if(
        || async move {
            let outcome = client.put(url, body).await;
            if let Err(error) = &outcome
                && is_missing_subscription_registration(error)
            {
                warn!(
                    "[Azure] {resource} rejected because Microsoft.Network is not registered on the subscription; re-registering before retrying."
                );
                request_network_provider_registration(client).await;
            }
            outcome
        },
        PROVIDER_REGISTRATION_RETRY_POLICY,
        is_missing_subscription_registration,
    )
    .await
    .map_err(|error| {
        if is_missing_subscription_registration(&error) {
            NetworkProvisioningError::CloudOperationTimedOut {
                operation: resource.to_string(),
            }
            .into()
        } else {
            error
        }
    })
}

fn is_missing_subscription_registration(error: &Error) -> bool {
    extract_arm_error_code(error) == Some(ArmErrorCode::MissingSubscriptionRegistration)
}
