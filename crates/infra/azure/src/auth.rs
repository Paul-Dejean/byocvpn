use std::sync::Arc;

use azure_core::credentials::{Secret, TokenCredential};
use azure_identity::ClientSecretCredential;
use log::debug;

use crate::error::Result;

const MANAGEMENT_SCOPE: &str = "https://management.azure.com/.default";

pub fn create_credential(
    tenant_id: &str,
    application_id: &str,
    secret_value: &str,
) -> Result<Arc<ClientSecretCredential>> {
    let secret = Secret::new(secret_value.to_string());
    Ok(ClientSecretCredential::new(
        tenant_id,
        application_id.to_string(),
        secret,
        None,
    )?)
}

pub async fn get_access_token(credential: &Arc<ClientSecretCredential>) -> Result<String> {
    debug!("[Azure] Acquiring management API access token...");
    let token_response = credential.get_token(&[MANAGEMENT_SCOPE], None).await?;
    debug!("[Azure] Access token acquired successfully.");
    Ok(token_response.token.secret().to_string())
}
