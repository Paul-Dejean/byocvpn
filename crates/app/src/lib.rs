pub mod provider_credentials;
pub mod stored_credentials_factory;

use std::sync::Arc;

use byocvpn_core::{app_context::AppContext, error::Result};

pub use provider_credentials::ProviderCredentials;
pub use stored_credentials_factory::StoredCredentialsProviderFactory;

pub fn open_app_context() -> Result<AppContext> {
    AppContext::open(Arc::new(StoredCredentialsProviderFactory))
}
