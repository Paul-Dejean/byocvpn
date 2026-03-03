use async_trait::async_trait;
use byocvpn_core::{
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, SpawnInstanceParams,
        TerminateInstanceParams,
    },
    commands::setup::Region,
    error::{NetworkProvisioningError, Result},
};
use serde_json::Value;

use crate::{
    auth::create_credential,
    client::AzureClient,
    instance, network,
};

/// Configuration required to create an `AzureProvider`.
pub struct AzureProviderConfig {
    /// Azure subscription ID.
    pub subscription_id: String,
    /// Azure Entra ID tenant (directory) ID.
    pub tenant_id: String,
    /// Service-principal client (application) ID.
    pub client_id: String,
    /// Service-principal client secret.
    pub client_secret: String,
}

/// Microsoft Azure implementation of `CloudProvider`.
///
/// Uses a service principal (`ClientSecretCredential`) for authentication
/// and the Azure Resource Manager REST API for all resource operations.
pub struct AzureProvider {
    client: AzureClient,
}

impl AzureProvider {
    pub fn new(config: AzureProviderConfig) -> Result<Self> {
        let credential = create_credential(
            &config.tenant_id,
            &config.client_id,
            &config.client_secret,
        )?;
        let client = AzureClient::new(credential, config.subscription_id);
        Ok(Self { client })
    }
}

#[async_trait]
impl CloudProvider for AzureProvider {
    fn get_provider_name(&self) -> CloudProviderName {
        CloudProviderName::Azure
    }

    async fn verify_permissions(&self) -> Result<Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }

    /// No global setup is required for Azure; all infrastructure is
    /// provisioned per-region by `enable_region`.
    async fn setup(&self) -> Result<()> {
        println!("[Azure] setup() — no global resources required.");
        Ok(())
    }

    /// Ensure the regional shared infrastructure (resource group, NSG, VNet,
    /// subnet) exists for `region`.
    async fn enable_region(&self, region: &str) -> Result<()> {
        network::ensure_resource_group(&self.client, region).await?;
        network::ensure_vnet_and_subnet(&self.client, region)
            .await
            .map_err(|error| NetworkProvisioningError::SubnetCreationFailed {
                reason: error.to_string(),
            })?;
        println!("[Azure] Region '{}' enabled.", region);
        Ok(())
    }

    async fn spawn_instance(&self, params: &SpawnInstanceParams) -> Result<InstanceInfo> {
        // Ensure regional infrastructure before spawning.
        instance::spawn_instance(&self.client, params.region, params).await
    }

    async fn terminate_instance(&self, params: &TerminateInstanceParams) -> Result<()> {
        instance::terminate_instance(&self.client, params.instance_id).await
    }

    async fn list_instances(&self, region: Option<&str>) -> Result<Vec<InstanceInfo>> {
        match region {
            Some(region_name) => instance::list_instances(&self.client, region_name).await,
            None => instance::list_all_instances(&self.client).await,
        }
    }

    async fn get_regions(&self) -> Result<Vec<Region>> {
        let region_list = network::list_regions(&self.client).await?;
        Ok(region_list
            .into_iter()
            .map(|(name, country)| Region { name, country })
            .collect())
    }
}
