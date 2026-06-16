use byocvpn_aws::AwsCredentials;
use byocvpn_azure::credentials::AzureCredentials;
use byocvpn_core::{
    cloud_provider::CloudProviderName, credentials::CredentialStore, error::Result,
};
use byocvpn_gcp::credentials::GcpCredentials;
use byocvpn_oracle::credentials::OracleCredentials;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderCredentials {
    Aws(AwsCredentials),
    Oracle(OracleCredentials),
    Gcp(GcpCredentials),
    Azure(AzureCredentials),
}

impl ProviderCredentials {
    pub fn load(provider: CloudProviderName, store: &CredentialStore) -> Result<Self> {
        match provider {
            CloudProviderName::Aws => AwsCredentials::from_store(store).map(Self::Aws),
            CloudProviderName::Oracle => OracleCredentials::from_store(store).map(Self::Oracle),
            CloudProviderName::Gcp => GcpCredentials::from_store(store).map(Self::Gcp),
            CloudProviderName::Azure => AzureCredentials::from_store(store).map(Self::Azure),
        }
    }

    pub fn write_to_store(&self, store: &mut CredentialStore) {
        match self {
            Self::Aws(credentials) => credentials.write_to_store(store),
            Self::Oracle(credentials) => credentials.write_to_store(store),
            Self::Gcp(credentials) => credentials.write_to_store(store),
            Self::Azure(credentials) => credentials.write_to_store(store),
        }
    }
}
