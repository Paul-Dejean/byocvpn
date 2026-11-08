pub mod provider_credentials;
pub mod stored_credentials_factory;

use std::sync::Arc;

use byocvpn_core::{
    app_context::AppContext,
    daemon_client::{DaemonClient, IpcDaemonClient},
    error::Result,
};

pub use provider_credentials::ProviderCredentials;
pub use stored_credentials_factory::StoredCredentialsProviderFactory;

pub fn open_app_context() -> Result<AppContext> {
    open_app_context_with_daemon_client(Arc::new(IpcDaemonClient))
}

pub fn open_app_context_with_daemon_client(
    daemon_client: Arc<dyn DaemonClient>,
) -> Result<AppContext> {
    AppContext::open(Arc::new(StoredCredentialsProviderFactory), daemon_client)
}
