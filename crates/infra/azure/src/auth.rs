use std::sync::Arc;

use azure_core::credentials::{Secret, TokenCredential};
use azure_identity::ClientSecretCredential;
use byocvpn_core::error::{CredentialsError, Result};
use log::*;

pub fn create_credential(
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<Arc<ClientSecretCredential>> {
    let secret = Secret::new(client_secret.to_string());
    ClientSecretCredential::new(tenant_id, client_id.to_string(), secret, None).map_err(|error| {
        CredentialsError::InvalidFormat {
            reason: format!(
                "Failed to create Azure service-principal credential: {}",
                error
            ),
        }
        .into()
    })
}

pub async fn get_access_token(credential: &Arc<ClientSecretCredential>) -> Result<String> {
    debug!("[Azure] Acquiring management API access token...");
    let token_response = credential
        .get_token(&["https://management.azure.com/.default"], None)
        .await
        .map_err(|error| CredentialsError::TokenAcquisitionFailed {
            provider: "Azure".to_string(),
            reason: error.to_string(),
        })?;

    debug!("[Azure] Access token acquired successfully.");
    Ok(token_response.token.secret().to_string())
}
