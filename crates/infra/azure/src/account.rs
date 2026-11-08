use byocvpn_core::{
    error::{Error, NetworkProvisioningError, Result},
    retry::{RetryPolicy, retry},
};
use log::{debug, info, warn};
use tokio::time::Duration;

use crate::{
    arm_resource::API_VERSION_RESOURCE_GROUPS,
    client::AzureClient,
    models::{EmptyRequest, ProviderRegistrationResponse},
};

const REQUIRED_NAMESPACES: &[&str] = &[
    "Microsoft.Network",
    "Microsoft.Compute",
    "Microsoft.Resources",
];

const PROVIDER_REGISTRATION_POLL_POLICY: RetryPolicy =
    RetryPolicy::fixed(60, Duration::from_secs(5));

pub async fn ensure_providers_registered(client: &AzureClient) -> Result<()> {
    for namespace in REQUIRED_NAMESPACES {
        let status_path = client.build_subscription_path(&format!("/providers/{namespace}"));
        let status_url = AzureClient::build_arm_url(&status_path, API_VERSION_RESOURCE_GROUPS);
        let registration_state = client
            .get::<ProviderRegistrationResponse>(&status_url)
            .await?
            .registration_state;

        if registration_state.as_deref() == Some("Registered") {
            continue;
        }

        let register_path =
            client.build_subscription_path(&format!("/providers/{namespace}/register"));
        let register_url = AzureClient::build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
        info!("[Azure] Registering provider '{namespace}'...");
        client
            .post::<_, ProviderRegistrationResponse>(&register_url, &EmptyRequest {})
            .await
            .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
                step: namespace.to_string(),
                reason: error.to_string(),
            })?;

        retry(
            || {
                let status_url = status_url.clone();
                async move {
                    let response: ProviderRegistrationResponse = client
                        .get(&status_url)
                        .await
                        .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
                            step: namespace.to_string(),
                            reason: error.to_string(),
                        })?;
                    let state = response.registration_state.as_deref().unwrap_or("Unknown");
                    debug!("[Azure] Provider '{namespace}' registration state: {state}");
                    if state == "Registered" {
                        Ok::<(), Error>(())
                    } else {
                        Err(NetworkProvisioningError::CloudOperationTimedOut {
                            operation: format!("{namespace} provider registration"),
                        }
                        .into())
                    }
                }
            },
            PROVIDER_REGISTRATION_POLL_POLICY,
        )
        .await?;
    }
    Ok(())
}

pub async fn request_network_provider_registration(client: &AzureClient) {
    let register_path = client.build_subscription_path("/providers/Microsoft.Network/register");
    let register_url = AzureClient::build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
    match client
        .post::<_, ProviderRegistrationResponse>(&register_url, &EmptyRequest {})
        .await
    {
        Ok(_) => debug!("[Azure] Microsoft.Network registration requested"),
        Err(error) => warn!("[Azure] Microsoft.Network registration request failed: {error}"),
    }
}
