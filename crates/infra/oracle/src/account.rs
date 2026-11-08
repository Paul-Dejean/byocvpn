use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    retry::{RetryPolicy, retry},
};
use log::info;
use serde_json::Value;
use tokio::time::Duration;

use crate::{
    client::OciClient,
    models::{RegionResponse, RegionSubscriptionResponse, SubscribeRegionRequest},
    network::{ensure_internet_gateway, ensure_vcn},
};

const READY_SUBSCRIPTION_STATUS: &str = "READY";
const REGION_SUBSCRIPTION_POLL_POLICY: RetryPolicy =
    RetryPolicy::fixed(40, Duration::from_secs(15));

pub async fn ensure_account_network(home_client: &OciClient) -> Result<()> {
    let vcn = ensure_vcn(home_client).await?;
    ensure_internet_gateway(home_client, &vcn).await?;
    info!(
        "[OCI] Home region {} network ready (VCN + internet gateway).",
        home_client.get_region()
    );
    Ok(())
}

pub async fn ensure_region_subscribed(home_client: &OciClient, region_name: &str) -> Result<()> {
    if is_region_ready(home_client, region_name).await? {
        return Ok(());
    }

    let region_key = find_region_key(home_client, region_name).await?;
    home_client
        .post::<_, Value>(
            &build_subscriptions_url(home_client),
            &SubscribeRegionRequest {
                region_key: region_key.clone(),
            },
        )
        .await
        .map_err(|error| NetworkProvisioningError::ProviderSetupFailed {
            step: format!("subscribe to region {region_name}"),
            reason: error.to_string(),
        })?;
    info!("[OCI] Subscribed to region {region_name} (key {region_key}), waiting for READY...");

    retry(
        || async move {
            if is_region_ready(home_client, region_name).await? {
                info!("[OCI] Region {region_name} is now READY.");
                return Ok(());
            }
            info!("[OCI] Region {region_name} not yet ready, retrying...");
            Err(NetworkProvisioningError::CloudOperationTimedOut {
                operation: format!("region {region_name} subscription"),
            }
            .into())
        },
        REGION_SUBSCRIPTION_POLL_POLICY,
    )
    .await
}

async fn is_region_ready(home_client: &OciClient, region_name: &str) -> Result<bool> {
    let subscriptions: Vec<RegionSubscriptionResponse> = home_client
        .get(&build_subscriptions_url(home_client))
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: error.to_string(),
        })?;
    Ok(subscriptions.iter().any(|subscription| {
        subscription.region_name == region_name && subscription.status == READY_SUBSCRIPTION_STATUS
    }))
}

async fn find_region_key(home_client: &OciClient, region_name: &str) -> Result<String> {
    let regions: Vec<RegionResponse> = home_client
        .get(&home_client.build_identity_url("/regions"))
        .await
        .map_err(|error| NetworkProvisioningError::NetworkQueryFailed {
            reason: error.to_string(),
        })?;
    regions
        .into_iter()
        .find(|region| region.name == region_name)
        .map(|region| region.key)
        .ok_or_else(|| {
            NetworkProvisioningError::ProviderSetupFailed {
                step: "region subscription".to_string(),
                reason: format!("region '{region_name}' not found"),
            }
            .into()
        })
}

fn build_subscriptions_url(home_client: &OciClient) -> String {
    home_client.build_identity_url(&format!(
        "/tenancies/{}/regionSubscriptions",
        home_client.get_compartment_ocid()
    ))
}
