use byocvpn_core::retry::{RetryPolicy, retry_if};
use serde::{Serialize, de::DeserializeOwned};
use tokio::time::Duration;
use tracing::{debug, warn};

use crate::{
    account::{NETWORK_PROVIDER_NAMESPACE, request_network_provider_registration},
    arm_resource::ArmResource,
    client::AzureClient,
    error::{ArmErrorCode, AzureError, Result},
};

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

pub async fn put_and_wait<B: Serialize + Sync>(
    client: &AzureClient,
    resource: &ArmResource,
    body: &B,
) -> Result<()> {
    let async_operation_url = put_with_provider_retry(client, resource, body).await?;
    if let Some(operation_url) = async_operation_url {
        debug!(%resource, "waiting for provisioning");
        client.wait_for_async_operation(&operation_url).await?;
    }
    Ok(())
}

pub async fn delete_resource(client: &AzureClient, resource: &ArmResource) {
    let url = client.build_resource_url(resource);
    debug!(%resource, "deleting");
    match client.delete(&url).await {
        Ok(Some(operation_url)) => {
            if let Err(error) = client.wait_for_async_operation(&operation_url).await {
                warn!(%resource, %error, "failed to wait for deletion");
            } else {
                debug!(%resource, "deleted");
            }
        }
        Ok(None) => debug!(%resource, "deleted"),
        Err(error) => warn!(%resource, %error, "failed to delete"),
    }
}

async fn put_with_provider_retry<B: Serialize + Sync>(
    client: &AzureClient,
    resource: &ArmResource,
    body: &B,
) -> Result<Option<String>> {
    let url = client.build_resource_url(resource);
    let url = url.as_str();
    let outcome = retry_if(
        || async move {
            let outcome = client.put(url, body).await;
            if let Err(error) = &outcome
                && is_missing_subscription_registration(error)
            {
                warn!(%resource, namespace = NETWORK_PROVIDER_NAMESPACE, "rejected because the provider is not registered, re-registering");
                request_network_provider_registration(client).await;
            }
            outcome
        },
        PROVIDER_REGISTRATION_RETRY_POLICY,
        is_missing_subscription_registration,
    )
    .await;

    match outcome {
        Err(error) if is_missing_subscription_registration(&error) => {
            Err(AzureError::ProviderRegistrationTimedOut {
                namespace: NETWORK_PROVIDER_NAMESPACE.to_string(),
            })
        }
        other => other,
    }
}

fn is_missing_subscription_registration(error: &AzureError) -> bool {
    error.get_arm_error_code() == Some(ArmErrorCode::MissingSubscriptionRegistration)
}
