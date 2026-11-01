use std::sync::Arc;

use crate::{
    error::Result, job_registry::SpawnJobRegistry, ledger::Ledger,
    provider_factory::ProviderFactory, provider_flags::ProviderFlags,
    server_monitor::UptimeNoticeTracker, settings::Settings,
};

pub struct AppContext {
    pub ledger: Ledger,
    pub settings: Settings,
    pub provider_flags: ProviderFlags,
    pub spawn_jobs: SpawnJobRegistry,
    pub uptime_notices: UptimeNoticeTracker,
    pub providers: Arc<dyn ProviderFactory>,
}

impl AppContext {
    pub fn open(providers: Arc<dyn ProviderFactory>) -> Result<Self> {
        Ok(Self {
            ledger: Ledger::open()?,
            settings: Settings::open()?,
            provider_flags: ProviderFlags::open()?,
            spawn_jobs: SpawnJobRegistry::default(),
            uptime_notices: UptimeNoticeTracker::default(),
            providers,
        })
    }
}
