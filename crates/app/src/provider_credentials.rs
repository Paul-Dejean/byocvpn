use byocvpn_aws::AwsCredentials;
use byocvpn_azure::credentials::AzureCredentials;
use byocvpn_core::{
    cloud_provider::CloudProviderName,
    credentials::{CredentialStore, StoredCredentials},
    error::Result,
};
use byocvpn_gcp::credentials::GcpCredentials;
use byocvpn_oracle::credentials::OracleCredentials;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "provider", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderCredentials {
    Aws(AwsCredentials),
    Oracle(OracleCredentials),
    Gcp(GcpCredentials),
    Azure(AzureCredentials),
}

impl ProviderCredentials {
    pub async fn load(provider: CloudProviderName) -> Result<Self> {
        let store = CredentialStore::load().await?;
        match provider {
            CloudProviderName::Aws => AwsCredentials::from_store(&store).map(Self::Aws),
            CloudProviderName::Oracle => OracleCredentials::from_store(&store).map(Self::Oracle),
            CloudProviderName::Gcp => GcpCredentials::from_store(&store).map(Self::Gcp),
            CloudProviderName::Azure => AzureCredentials::from_store(&store).map(Self::Azure),
        }
    }

    pub async fn save(&self) -> Result<()> {
        let mut store = CredentialStore::load().await?;
        match self {
            Self::Aws(credentials) => credentials.write_to_store(&mut store),
            Self::Oracle(credentials) => credentials.write_to_store(&mut store),
            Self::Gcp(credentials) => credentials.write_to_store(&mut store),
            Self::Azure(credentials) => credentials.write_to_store(&mut store),
        }
        store.save()
    }

    pub async fn delete(provider: CloudProviderName) -> Result<()> {
        let mut store = CredentialStore::load().await?;
        let section = match provider {
            CloudProviderName::Aws => AwsCredentials::CREDENTIALS_SECTION,
            CloudProviderName::Oracle => OracleCredentials::CREDENTIALS_SECTION,
            CloudProviderName::Gcp => GcpCredentials::CREDENTIALS_SECTION,
            CloudProviderName::Azure => AzureCredentials::CREDENTIALS_SECTION,
        };
        store.delete_section(section);
        store.save()
    }
}
