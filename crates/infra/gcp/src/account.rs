use byocvpn_core::retry::RetryPolicy;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use crate::{
    client::GcpClient,
    error::{GcpError, Result},
    network::{ensure_firewall_rules, ensure_vpc},
    operations::wait_for_service_usage_operation,
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
    let service: ServiceResponse = client.get(&service_url).await?;
    if service.state.as_deref() == Some(ENABLED_SERVICE_STATE) {
        return Ok(());
    }

    info!(
        "[GCP] Enabling {api_service} for project '{}'...",
        client.project_id
    );
    let operation: ServiceUsageOperationReference = client
        .post(&format!("{service_url}:enable"), &EmptyRequest {})
        .await?;
    let operation_name =
        operation
            .name
            .as_deref()
            .ok_or_else(|| GcpError::MissingOperationName {
                api_service: api_service.to_string(),
            })?;

    let operation_url = GcpClient::build_service_usage_url(operation_name);
    wait_for_service_usage_operation(client, &operation_url, SERVICE_USAGE_OPERATION_POLL_POLICY)
        .await?;
    info!(
        "[GCP] {api_service} enabled for project '{}'.",
        client.project_id
    );
    Ok(())
}

#[derive(Deserialize)]
struct ServiceResponse {
    state: Option<String>,
}

#[derive(Deserialize)]
struct ServiceUsageOperationReference {
    name: Option<String>,
}

#[derive(Serialize)]
struct EmptyRequest {}
