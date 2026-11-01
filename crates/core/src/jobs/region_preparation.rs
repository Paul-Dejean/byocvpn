use log::info;

use crate::{
    app_context::AppContext,
    cloud_provider::CloudProvider,
    error::Result,
    jobs::{
        enable_region::{EnableRegionObserver, run_enable_region_job},
        provision_account::{ProvisionAccountObserver, run_provision_account_job},
    },
};

pub async fn ensure_region_ready<Observer>(
    provider: &dyn CloudProvider,
    context: &AppContext,
    region: &str,
    observer: &Observer,
) -> Result<()>
where
    Observer: ProvisionAccountObserver + EnableRegionObserver,
{
    let provider_name = provider.get_provider_name();

    if context.provider_flags.is_provisioned(&provider_name) {
        info!("{provider_name} account already provisioned");
    } else {
        run_provision_account_job(provider, context, observer).await?;
    }

    if context
        .provider_flags
        .is_region_enabled(&provider_name, region)
    {
        info!("{provider_name} region {region} already enabled");
    } else {
        run_enable_region_job(provider, context, region, observer).await?;
    }

    Ok(())
}
