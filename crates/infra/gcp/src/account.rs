use byocvpn_core::{
    error::{NetworkProvisioningError, Result},
    retry::RetryPolicy,
};
use log::info;
use tokio::time::Duration;

use crate::{
    client::GcpClient,
    models::{EmptyRequest, Operation, ServiceResponse},
    network::{ensure_firewall_rules, ensure_vpc},
    operations::wait_for_operation,
};

const COMPUTE_API_SERVICE: &str = "compute.googleapis.com";
const CLOUD_RESOURCE_MANAGER_API_SERVICE: &str = "cloudresourcemanager.googleapis.com";
const CLOUD_BILLING_API_SERVICE: &str = "cloudbilling.googleapis.com";
const ENABLED_SERVICE_STATE: &str = "ENABLED";
const SERVICE_USAGE_OPERATION_POLL_POLICY: RetryPolicy =
    RetryPolicy::fixed(30, Duration::from_secs(5));

pub async fn ensure_account_network(client: &GcpClient) -> Result<()> {
    ensure_compute_api_enabled(client).await?;
    ensure_vpc(client).await?;
    ensure_firewall_rules(client).await?;
    info!("[GCP] Account network ready (VPC + firewall).");
    Ok(())
}

pub async fn ensure_compute_api_enabled(client: &GcpClient) -> Result<()> {
    ensure_api_enabled(client, COMPUTE_API_SERVICE).await
}

pub async fn ensure_cloud_resource_manager_api_enabled(client: &GcpClient) -> Result<()> {
    ensure_api_enabled(client, CLOUD_RESOURCE_MANAGER_API_SERVICE).await
}

pub async fn ensure_cloud_billing_api_enabled(client: &GcpClient) -> Result<()> {
    ensure_api_enabled(client, CLOUD_BILLING_API_SERVICE).await
}

async fn ensure_api_enabled(client: &GcpClient, api_service: &str) -> Result<()> {
    let service_url = GcpClient::build_service_usage_url(&format!(
        "projects/{}/services/{api_service}",
        client.project_id
    ));
    let service: ServiceResponse = client.get(&service_url).await.map_err(|error| {
        NetworkProvisioningError::ProviderSetupFailed {
            step: format!("{api_service} API check"),
            reason: error.to_string(),
        }
    })?;
    if service.state.as_deref() == Some(ENABLED_SERVICE_STATE) {
        return Ok(());
    }

    info!(
        "[GCP] Enabling {api_service} for project '{}'...",
        client.project_id
    );
    let build_enablement_error = |reason: String| NetworkProvisioningError::ProviderSetupFailed {
        step: format!("{api_service} API enablement"),
        reason,
    };
    let operation: Operation = client
        .post(&format!("{service_url}:enable"), &EmptyRequest {})
        .await
        .map_err(|error| build_enablement_error(error.to_string()))?;
    let operation_name = operation.name.as_deref().ok_or_else(|| {
        build_enablement_error("operation response missing 'name' field".to_string())
    })?;

    let operation_url = GcpClient::build_service_usage_url(operation_name);
    wait_for_operation(client, &operation_url, SERVICE_USAGE_OPERATION_POLL_POLICY).await?;
    info!(
        "[GCP] {api_service} enabled for project '{}'.",
        client.project_id
    );
    Ok(())
}
