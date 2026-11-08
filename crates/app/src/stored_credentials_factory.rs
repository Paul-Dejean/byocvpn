use async_trait::async_trait;
use byocvpn_aws::{AwsCredentials, AwsProvider};
use byocvpn_azure::{AzureCredentials, AzureProvider};
use byocvpn_core::{
    cloud_provider::{CloudProvider, CloudProviderName},
    credentials::{CredentialStore, StoredCredentials},
    error::Result,
    provider_factory::ProviderFactory,
};
use byocvpn_gcp::{GcpCredentials, GcpProvider};
use byocvpn_oracle::{OracleCredentials, OracleProvider};
use log::debug;

const ALL_PROVIDERS: [CloudProviderName; 4] = [
    CloudProviderName::Aws,
    CloudProviderName::Azure,
    CloudProviderName::Gcp,
    CloudProviderName::Oracle,
];

pub struct StoredCredentialsProviderFactory;

#[async_trait]
impl ProviderFactory for StoredCredentialsProviderFactory {
    async fn list_configured(&self) -> Result<Vec<CloudProviderName>> {
        let store = CredentialStore::load().await?;
        Ok(ALL_PROVIDERS
            .into_iter()
            .filter(|provider| has_credentials(provider, &store))
            .collect())
    }

    async fn create(&self, provider: CloudProviderName) -> Result<Box<dyn CloudProvider>> {
        debug!("Creating {provider} cloud provider");
        let store = CredentialStore::load().await?;
        let cloud_provider: Box<dyn CloudProvider> = match provider {
            CloudProviderName::Aws => {
                Box::new(AwsProvider::new(AwsCredentials::from_store(&store)?))
            }
            CloudProviderName::Azure => {
                Box::new(AzureProvider::new(AzureCredentials::from_store(&store)?)?)
            }
            CloudProviderName::Gcp => {
                Box::new(GcpProvider::connect(GcpCredentials::from_store(&store)?).await?)
            }
            CloudProviderName::Oracle => {
                Box::new(OracleProvider::new(OracleCredentials::from_store(&store)?))
            }
        };
        Ok(cloud_provider)
    }
}

fn has_credentials(provider: &CloudProviderName, store: &CredentialStore) -> bool {
    match provider {
        CloudProviderName::Aws => AwsCredentials::from_store(store).is_ok(),
        CloudProviderName::Azure => AzureCredentials::from_store(store).is_ok(),
        CloudProviderName::Gcp => GcpCredentials::from_store(store).is_ok(),
        CloudProviderName::Oracle => OracleCredentials::from_store(store).is_ok(),
    }
}
