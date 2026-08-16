use std::{collections::HashMap, sync::Mutex};

use byocvpn_core::cloud_provider::{CloudProviderName, SpawnJob, SpawnStep, SpawnStepStatus};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpawnJobStatus {
    Running,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnJobStep {
    pub id: String,
    pub label: String,
    pub status: SpawnStepStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnJobState {
    pub job_id: String,
    pub region: String,
    pub provider: CloudProviderName,
    pub instance_id: Option<String>,
    pub status: SpawnJobStatus,
    pub error: Option<String>,
    pub steps: Vec<SpawnJobStep>,
}

pub struct SpawnJobRegistry(Mutex<HashMap<String, SpawnJobState>>);

impl SpawnJobRegistry {
    pub fn new() -> Self {
        SpawnJobRegistry(Mutex::new(HashMap::new()))
    }

    pub fn list(&self) -> Vec<SpawnJobState> {
        self.0
            .lock()
            .map(|registry| registry.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn register(&self, job: SpawnJob) -> SpawnJobState {
        let SpawnJob {
            job_id,
            steps,
            region,
            provider,
        } = job;
        let state = SpawnJobState {
            job_id: job_id.clone(),
            region,
            provider,
            instance_id: None,
            status: SpawnJobStatus::Running,
            error: None,
            steps: steps.iter().map(build_pending_step).collect(),
        };

        if let Ok(mut registry) = self.0.lock() {
            registry.insert(job_id, state.clone());
        }

        state
    }

    pub fn update_step_status(
        &self,
        job_id: &str,
        step_id: &str,
        status: SpawnStepStatus,
        error: Option<String>,
    ) {
        if let Ok(mut registry) = self.0.lock() {
            if let Some(job) = registry.get_mut(job_id) {
                if let Some(step) = job.steps.iter_mut().find(|step| step.id == step_id) {
                    step.status = status;
                    step.error = error;
                }
            }
        }
    }

    pub fn set_instance_id(&self, job_id: &str, instance_id: String) {
        if let Ok(mut registry) = self.0.lock() {
            if let Some(job) = registry.get_mut(job_id) {
                job.instance_id = Some(instance_id);
            }
        }
    }

    pub fn mark_failed(&self, job_id: &str, error: String) {
        if let Ok(mut registry) = self.0.lock() {
            if let Some(job) = registry.get_mut(job_id) {
                job.status = SpawnJobStatus::Failed;
                job.error = Some(error);
            }
        }
    }

    pub fn instance_ids_in_progress(&self) -> Vec<String> {
        self.0
            .lock()
            .map(|registry| {
                registry
                    .values()
                    .filter(|job| job.status == SpawnJobStatus::Running)
                    .filter_map(|job| job.instance_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn deregister(&self, job_id: &str) {
        if let Ok(mut registry) = self.0.lock() {
            registry.remove(job_id);
        }
    }
}

fn build_pending_step(step: &SpawnStep) -> SpawnJobStep {
    SpawnJobStep {
        id: step.id.clone(),
        label: step.label.clone(),
        status: SpawnStepStatus::Pending,
        error: None,
    }
}
