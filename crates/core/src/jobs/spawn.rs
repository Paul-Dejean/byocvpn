#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use async_trait::async_trait;
use log::{info, warn};
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    app_context::AppContext,
    cloud_provider::{
        CloudProvider, CloudProviderName, InstanceInfo, InstanceState, JobStep, JobStepStatus,
        SpawnInstanceParams, SpawnJob,
    },
    config::{generate_client_config, get_wireguard_config_file_path},
    connectivity,
    crypto::generate_keypair,
    error::{ComputeProvisioningError, ConfigurationError, Error, ErrorPayload, Result},
    job_registry::{SpawnJobRegistry, SpawnJobState},
    jobs::{JobObserver, JobSteps, build_job_id, run_job_steps},
    ledger::LedgerEntry,
};

const LAUNCH_STEP_ID: &str = "launch";
const WIREGUARD_READY_STEP_ID: &str = "wireguard_ready";

pub trait SpawnJobObserver: JobObserver {
    fn on_job_started(&self, job: &SpawnJobState);
    fn on_instance_launched(&self, instance: &InstanceInfo);
    fn on_job_completed(&self, instance: &InstanceInfo);
    fn on_job_failed(&self, error: &Error);
}

pub async fn run_spawn_job(
    provider: &dyn CloudProvider,
    context: &AppContext,
    region: &str,
    observer: &dyn SpawnJobObserver,
) -> Result<InstanceInfo> {
    let provider_name = provider.get_provider_name();
    let job = SpawnJob {
        job_id: build_job_id(&provider_name, None),
        steps: provider.get_spawn_steps(region),
        region: region.to_string(),
        provider: provider_name,
    };
    let job_state = context.spawn_jobs.register(job.clone());
    observer.on_job_started(&job_state);

    let mut spawn_steps = SpawnSteps {
        provider,
        context,
        job: &job,
        keys: SpawnKeys::generate(),
        launched_instance: None,
        observer,
    };
    let tracking_observer = RegistryTrackingObserver {
        registry: &context.spawn_jobs,
        job_id: &job.job_id,
        observer,
    };

    let outcome = run_job_steps(&job.steps, &mut spawn_steps, &tracking_observer).await;
    let outcome = outcome.and_then(|()| {
        spawn_steps.launched_instance.ok_or_else(|| {
            ComputeProvisioningError::LaunchStepMissing {
                step: "completion".to_string(),
            }
            .into()
        })
    });

    match outcome {
        Ok(mut instance) => {
            context.spawn_jobs.deregister(&job.job_id);
            context.ledger.mark_setup_complete(&instance.id);
            instance.state = InstanceState::Running;
            info!(
                "Spawn job {} completed: instance {}",
                job.job_id, instance.id
            );
            observer.on_job_completed(&instance);
            Ok(instance)
        }
        Err(error) => {
            context
                .spawn_jobs
                .mark_failed(&job.job_id, error.to_payload());
            warn!("Spawn job {} failed: {error}", job.job_id);
            observer.on_job_failed(&error);
            Err(error)
        }
    }
}

struct SpawnSteps<'a> {
    provider: &'a dyn CloudProvider,
    context: &'a AppContext,
    job: &'a SpawnJob,
    keys: SpawnKeys,
    launched_instance: Option<InstanceInfo>,
    observer: &'a dyn SpawnJobObserver,
}

#[async_trait]
impl JobSteps for SpawnSteps<'_> {
    async fn run_step(&mut self, step: &JobStep) -> Result<()> {
        match step.id.as_str() {
            LAUNCH_STEP_ID => {
                let instance = launch_instance(self.provider, self.job, &self.keys).await?;
                self.context
                    .spawn_jobs
                    .set_instance_id(&self.job.job_id, instance.id.clone());
                self.context
                    .ledger
                    .insert_entry(LedgerEntry::build_for_launched_instance(&instance));
                self.launched_instance = Some(instance);
                Ok(())
            }
            WIREGUARD_READY_STEP_ID => {
                wait_until_wireguard_ready(self.launched_instance.as_ref()).await
            }
            step_id => {
                self.provider
                    .run_spawn_step(step_id, &self.job.region)
                    .await
            }
        }
    }

    fn on_step_completed(&mut self, step: &JobStep) {
        if step.id == LAUNCH_STEP_ID
            && let Some(instance) = &self.launched_instance
        {
            self.observer.on_instance_launched(instance);
        }
    }
}

struct RegistryTrackingObserver<'a> {
    registry: &'a SpawnJobRegistry,
    job_id: &'a str,
    observer: &'a dyn SpawnJobObserver,
}

impl JobObserver for RegistryTrackingObserver<'_> {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, error: Option<ErrorPayload>) {
        self.registry
            .update_step_status(self.job_id, &step.id, status.clone(), error.clone());
        self.observer.on_step_changed(step, status, error);
    }
}

async fn launch_instance(
    provider: &dyn CloudProvider,
    job: &SpawnJob,
    keys: &SpawnKeys,
) -> Result<InstanceInfo> {
    let params = SpawnInstanceParams {
        region: &job.region,
        server_private_key: &keys.server_private_key,
        client_public_key: &keys.client_public_key,
        spawn_id: &job.job_id,
    };

    let mut instance = provider.spawn_instance(&params).await?;
    info!("Spawned instance: {}", instance.id);

    write_wireguard_config(
        &job.provider,
        &job.region,
        &instance,
        &keys.client_private_key,
        &keys.server_public_key,
    )
    .await?;

    instance.state = InstanceState::Installing;
    Ok(instance)
}

async fn wait_until_wireguard_ready(launched_instance: Option<&InstanceInfo>) -> Result<()> {
    let instance =
        launched_instance.ok_or_else(|| ComputeProvisioningError::LaunchStepMissing {
            step: WIREGUARD_READY_STEP_ID.to_string(),
        })?;
    connectivity::wait_until_ready(&instance.public_ip_v4).await
}

async fn write_wireguard_config(
    provider_name: &CloudProviderName,
    region: &str,
    instance: &InstanceInfo,
    client_private_key: &str,
    server_public_key: &str,
) -> Result<()> {
    let client_config = generate_client_config(
        client_private_key,
        server_public_key,
        &instance.public_ip_v4,
    )?;

    let wireguard_file_path =
        get_wireguard_config_file_path(provider_name, region, &instance.id).await?;

    let mut file = fs::File::create(wireguard_file_path.clone())
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to create config file: {error}"),
        })?;
    file.write_all(client_config.as_bytes())
        .await
        .map_err(|error| ConfigurationError::TunnelConfiguration {
            reason: format!("failed to write config file: {error}"),
        })?;

    #[cfg(unix)]
    {
        let metadata = fs::metadata(wireguard_file_path.clone())
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to read config file metadata: {error}"),
            })?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(wireguard_file_path.clone(), perms)
            .await
            .map_err(|error| ConfigurationError::TunnelConfiguration {
                reason: format!("failed to set config file permissions: {error}"),
            })?;
    }

    if let Some(path_str) = wireguard_file_path.to_str() {
        info!("Client config written to {path_str}");
    }

    Ok(())
}

struct SpawnKeys {
    client_private_key: String,
    client_public_key: String,
    server_private_key: String,
    server_public_key: String,
}

impl SpawnKeys {
    fn generate() -> Self {
        let (client_private_key, client_public_key) = generate_keypair();
        let (server_private_key, server_public_key) = generate_keypair();
        Self {
            client_private_key,
            client_public_key,
            server_private_key,
            server_public_key,
        }
    }
}
