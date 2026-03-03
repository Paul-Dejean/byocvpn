/// Azure service-principal authentication via the official `azure_identity` library.
///
/// Uses `ClientSecretCredential` (tenant ID + client ID + client secret) to
/// obtain short-lived Bearer tokens for the Azure Resource Manager (ARM) API.
use azure_core::credentials::{Secret, TokenCredential};
use azure_identity::ClientSecretCredential;
use byocvpn_core::error::{ConfigurationError, Result};
use std::sync::Arc;

/// Build a `ClientSecretCredential` from service-principal components.
///
/// The returned `Arc` is cheap to clone and safe to share across threads;
/// the credential handles token caching and renewal internally.
pub fn create_credential(
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<Arc<ClientSecretCredential>> {
    let secret = Secret::new(client_secret.to_string());
    ClientSecretCredential::new(tenant_id, client_id.to_string(), secret, None).map_err(
        |error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to create Azure service-principal credential: {}",
                error
            ))
            .into()
        },
    )
}

/// Obtain a short-lived Bearer token scoped to the ARM management plane.
///
/// The `azure_identity` library caches tokens and refreshes them before
/// expiry, so repeated calls are inexpensive.
pub async fn bearer_token(credential: &Arc<ClientSecretCredential>) -> Result<String> {
    let token_response = credential
        .get_token(&["https://management.azure.com/.default"], None)
        .await
        .map_err(|error| {
            ConfigurationError::InvalidCloudProvider(format!(
                "Failed to obtain Azure access token: {}",
                error
            ))
        })?;

    Ok(token_response.token.secret().to_string())
}
