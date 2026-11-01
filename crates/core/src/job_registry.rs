use std::{
    collections::HashMap,
    sync::{Mutex, PoisonError},
};

use serde::{Deserialize, Serialize};

use crate::{
    cloud_provider::{CloudProviderName, JobStep, JobStepStatus, SpawnJob},
    error::ErrorPayload,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpawnJobStatus {
    Running,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnJobStep {
    pub id: String,
    pub label: String,
    pub status: JobStepStatus,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnJobState {
    pub job_id: String,
    pub region: String,
    pub provider: CloudProviderName,
    pub instance_id: Option<String>,
    pub status: SpawnJobStatus,
    pub error: Option<ErrorPayload>,
    pub steps: Vec<SpawnJobStep>,
}

#[derive(Default)]
pub struct SpawnJobRegistry {
    jobs: Mutex<HashMap<String, SpawnJobState>>,
}

impl SpawnJobRegistry {
    pub fn list(&self) -> Vec<SpawnJobState> {
        self.lock_jobs().values().cloned().collect()
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
        self.lock_jobs().insert(job_id, state.clone());
        state
    }

    pub fn update_step_status(
        &self,
        job_id: &str,
        step_id: &str,
        status: JobStepStatus,
        error: Option<ErrorPayload>,
    ) {
        if let Some(job) = self.lock_jobs().get_mut(job_id)
            && let Some(step) = job.steps.iter_mut().find(|step| step.id == step_id)
        {
            step.status = status;
            step.error = error;
        }
    }

    pub fn set_instance_id(&self, job_id: &str, instance_id: String) {
        if let Some(job) = self.lock_jobs().get_mut(job_id) {
            job.instance_id = Some(instance_id);
        }
    }

    pub fn mark_failed(&self, job_id: &str, error: ErrorPayload) {
        if let Some(job) = self.lock_jobs().get_mut(job_id) {
            job.status = SpawnJobStatus::Failed;
            job.error = Some(error);
        }
    }

    pub fn instance_ids_in_progress(&self) -> Vec<String> {
        self.lock_jobs()
            .values()
            .filter(|job| job.status == SpawnJobStatus::Running)
            .filter_map(|job| job.instance_id.clone())
            .collect()
    }

    pub fn deregister(&self, job_id: &str) {
        self.lock_jobs().remove(job_id);
    }

    fn lock_jobs(&self) -> std::sync::MutexGuard<'_, HashMap<String, SpawnJobState>> {
        self.jobs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn build_pending_step(step: &JobStep) -> SpawnJobStep {
    SpawnJobStep {
        id: step.id.clone(),
        label: step.label.clone(),
        status: JobStepStatus::Pending,
        error: None,
    }
}
