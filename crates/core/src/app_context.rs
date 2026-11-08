use std::sync::Arc;

use crate::{
    daemon_client::DaemonClient, error::Result, job_registry::SpawnJobRegistry, ledger::Ledger,
    pricing::PricingCache, provider_factory::ProviderFactory, provider_flags::ProviderFlags,
    server_monitor::UptimeNoticeTracker, settings::Settings,
};

pub struct AppContext {
    pub ledger: Ledger,
    pub settings: Settings,
    pub provider_flags: ProviderFlags,
    pub pricing_cache: PricingCache,
    pub spawn_jobs: SpawnJobRegistry,
    pub uptime_notices: UptimeNoticeTracker,
    pub providers: Arc<dyn ProviderFactory>,
    pub daemon_client: Arc<dyn DaemonClient>,
}

impl AppContext {
    pub fn open(
        providers: Arc<dyn ProviderFactory>,
        daemon_client: Arc<dyn DaemonClient>,
    ) -> Result<Self> {
        Ok(Self {
            ledger: Ledger::open()?,
            settings: Settings::open()?,
            provider_flags: ProviderFlags::open()?,
            pricing_cache: PricingCache::open()?,
            spawn_jobs: SpawnJobRegistry::default(),
            uptime_notices: UptimeNoticeTracker::default(),
            providers,
            daemon_client,
        })
    }
}
