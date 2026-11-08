use byocvpn_core::retry::{RetryPolicy, retry};
use log::{debug, info, warn};
use tokio::time::Duration;

use crate::{
    arm_resource::API_VERSION_RESOURCE_GROUPS,
    client::AzureClient,
    error::{AzureError, Result},
    models::{EmptyRequest, ProviderRegistrationResponse},
};

pub const NETWORK_PROVIDER_NAMESPACE: &str = "Microsoft.Network";
const REQUIRED_NAMESPACES: &[&str] = &[
    NETWORK_PROVIDER_NAMESPACE,
    "Microsoft.Compute",
    "Microsoft.Resources",
];
const REGISTERED_STATE: &str = "Registered";

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

        if registration_state.as_deref() == Some(REGISTERED_STATE) {
            continue;
        }

        let register_path =
            client.build_subscription_path(&format!("/providers/{namespace}/register"));
        let register_url = AzureClient::build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
        info!("[Azure] Registering provider '{namespace}'...");
        client
            .post::<_, ProviderRegistrationResponse>(&register_url, &EmptyRequest {})
            .await?;

        retry(
            || {
                let status_url = status_url.clone();
                async move {
                    let response: ProviderRegistrationResponse = client.get(&status_url).await?;
                    let state = response.registration_state.as_deref().unwrap_or("Unknown");
                    debug!("[Azure] Provider '{namespace}' registration state: {state}");
                    if state == REGISTERED_STATE {
                        Ok(())
                    } else {
                        Err(AzureError::ProviderRegistrationTimedOut {
                            namespace: (*namespace).to_string(),
                        })
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
    let register_path = client
        .build_subscription_path(&format!("/providers/{NETWORK_PROVIDER_NAMESPACE}/register"));
    let register_url = AzureClient::build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
    match client
        .post::<_, ProviderRegistrationResponse>(&register_url, &EmptyRequest {})
        .await
    {
        Ok(_) => debug!("[Azure] {NETWORK_PROVIDER_NAMESPACE} registration requested"),
        Err(error) => {
            warn!("[Azure] {NETWORK_PROVIDER_NAMESPACE} registration request failed: {error}");
        }
    }
}
