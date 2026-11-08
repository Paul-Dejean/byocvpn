use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use log::{info, warn};
use tokio::{fs, task::JoinSet};

use crate::{
    app_context::AppContext,
    cloud_provider::{CloudProviderName, InstanceInfo, InstanceState, TerminateInstanceParams},
    config::get_wireguard_config_file_path,
    connectivity::{self, ProbeStatus},
    daemon_client::fetch_vpn_status,
    error::{ComputeProvisioningError, ConfigurationError, Error, Result},
    provider_factory::ProviderFactory,
    session,
};

pub struct InstanceListing {
    pub instances: Vec<InstanceInfo>,
    pub failures: Vec<ProviderListingFailure>,
}

pub struct ProviderListingFailure {
    pub provider: CloudProviderName,
    pub error: Error,
}

pub async fn list_instances(
    context: &AppContext,
    provider_filter: Option<CloudProviderName>,
    region: Option<&str>,
) -> Result<InstanceListing> {
    let provider_names = match provider_filter {
        Some(provider_name) => vec![provider_name],
        None => context.providers.list_configured().await?,
    };

    let mut listing_tasks = JoinSet::new();
    for provider_name in provider_names.clone() {
        let factory = Arc::clone(&context.providers);
        let region = region.map(str::to_string);
        listing_tasks.spawn(async move {
            let outcome = list_provider_instances(&*factory, provider_name.clone(), region).await;
            (provider_name, outcome)
        });
    }

    let mut outcomes = HashMap::new();
    while let Some(joined) = listing_tasks.join_next().await {
        match joined {
            Ok((provider_name, outcome)) => {
                outcomes.insert(provider_name, outcome);
            }
            Err(error) => warn!("Instance listing task failed: {error}"),
        }
    }

    let mut instances = Vec::new();
    let mut queried_providers = Vec::new();
    let mut failures = Vec::new();
    for provider_name in provider_names {
        match outcomes.remove(&provider_name) {
            Some(Ok(found)) => {
                queried_providers.push(provider_name);
                instances.extend(found);
            }
            Some(Err(error)) => failures.push(ProviderListingFailure {
                provider: provider_name,
                error,
            }),
            None => {}
        }
    }

    let running_ids: HashSet<&str> = instances
        .iter()
        .map(|instance| instance.id.as_str())
        .collect();
    context
        .ledger
        .reconcile_terminated(&running_ids, &queried_providers);

    refresh_instance_states(context, &mut instances).await;

    Ok(InstanceListing {
        instances,
        failures,
    })
}

pub async fn require_instance(context: &AppContext, instance_id: &str) -> Result<InstanceInfo> {
    find_instance(context, instance_id).await?.ok_or_else(|| {
        ComputeProvisioningError::InstanceNotFound {
            instance_id: instance_id.to_string(),
        }
        .into()
    })
}

pub async fn find_instance(
    context: &AppContext,
    instance_id: &str,
) -> Result<Option<InstanceInfo>> {
    let listing = list_instances(context, None, None).await?;
    Ok(listing
        .instances
        .into_iter()
        .find(|instance| instance.id == instance_id))
}

pub async fn terminate_instance(
    context: &AppContext,
    provider_name: CloudProviderName,
    region: &str,
    instance_id: &str,
) -> Result<()> {
    disconnect_if_connected_to(context, instance_id).await?;

    let provider = context.providers.create(provider_name.clone()).await?;
    provider
        .terminate_instance(&TerminateInstanceParams {
            region,
            instance_id,
        })
        .await?;

    let wireguard_file_path =
        get_wireguard_config_file_path(&provider_name, region, instance_id).await?;
    if fs::metadata(&wireguard_file_path).await.is_ok() {
        fs::remove_file(&wireguard_file_path)
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to remove config file: {error}"),
            })?;
    }

    context.ledger.mark_terminated(instance_id);
    info!("Terminated instance: {instance_id}");
    Ok(())
}

async fn disconnect_if_connected_to(context: &AppContext, instance_id: &str) -> Result<()> {
    let status = fetch_vpn_status(context.daemon_client.as_ref()).await?;
    let is_connected_to_instance = status.connected
        && status
            .instance
            .is_some_and(|connected| connected.instance_id == instance_id);
    if is_connected_to_instance {
        info!("Disconnecting from {instance_id} before terminating it");
        session::disconnect(context.daemon_client.as_ref()).await?;
    }
    Ok(())
}

async fn list_provider_instances(
    factory: &dyn ProviderFactory,
    provider_name: CloudProviderName,
    region: Option<String>,
) -> Result<Vec<InstanceInfo>> {
    let provider = factory.create(provider_name).await?;
    provider.list_instances(region.as_deref()).await
}

async fn refresh_instance_states(context: &AppContext, instances: &mut [InstanceInfo]) {
    let in_progress_ids = context.spawn_jobs.instance_ids_in_progress();

    let mut probe_tasks = JoinSet::new();
    for instance in instances.iter() {
        if instance.state != InstanceState::Running || in_progress_ids.contains(&instance.id) {
            continue;
        }
        let instance_id = instance.id.clone();
        let instance_ip = instance.public_ip_v4.clone();
        probe_tasks
            .spawn(async move { (instance_id, connectivity::probe_status(&instance_ip).await) });
    }

    let mut probe_results = HashMap::new();
    while let Some(joined) = probe_tasks.join_next().await {
        if let Ok((instance_id, status)) = joined {
            probe_results.insert(instance_id, status);
        }
    }

    for instance in instances.iter_mut() {
        if in_progress_ids.contains(&instance.id) {
            instance.state = InstanceState::Installing;
            continue;
        }
        match probe_results.remove(&instance.id) {
            Some(ProbeStatus::Ready) => {
                context.ledger.mark_setup_complete(&instance.id);
                instance.state = InstanceState::Running;
            }
            Some(ProbeStatus::Error(reason)) => {
                instance.state = InstanceState::Error;
                instance.error_reason = Some(reason);
            }
            Some(ProbeStatus::Installing) => {
                instance.state = InstanceState::Installing;
            }
            None => {}
        }
    }
}
