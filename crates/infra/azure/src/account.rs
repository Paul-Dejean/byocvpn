use byocvpn_core::retry::{RetryPolicy, retry};
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::{
    arm_resource::API_VERSION_RESOURCE_GROUPS,
    client::AzureClient,
    error::{AzureError, Result},
    models::{EmptyRequest, ProviderRegistrationResponse},
    state::AzureRegistrationState,
};

pub const NETWORK_PROVIDER_NAMESPACE: &str = "Microsoft.Network";
const REQUIRED_NAMESPACES: &[&str] = &[
    NETWORK_PROVIDER_NAMESPACE,
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

        if registration_state == Some(AzureRegistrationState::Registered) {
            continue;
        }

        let register_path =
            client.build_subscription_path(&format!("/providers/{namespace}/register"));
        let register_url = AzureClient::build_arm_url(&register_path, API_VERSION_RESOURCE_GROUPS);
        info!(%namespace, "registering resource provider");
        client
            .post::<_, ProviderRegistrationResponse>(&register_url, &EmptyRequest {})
            .await?;

        retry(
            || {
                let status_url = status_url.clone();
                async move {
                    let response: ProviderRegistrationResponse = client.get(&status_url).await?;
                    match response.registration_state {
                        Some(AzureRegistrationState::Registered) => Ok(()),
                        other => {
                            let state = other
                                .map_or_else(|| "unknown".to_string(), |state| state.to_string());
                            debug!(%namespace, %state, "resource provider registration state");
                            Err(AzureError::ProviderRegistrationTimedOut {
                                namespace: (*namespace).to_string(),
                            })
                        }
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
        Ok(_) => debug!(
            namespace = NETWORK_PROVIDER_NAMESPACE,
            "registration requested"
        ),
        Err(error) => {
            warn!(namespace = NETWORK_PROVIDER_NAMESPACE, %error, "registration request failed");
        }
    }
}
