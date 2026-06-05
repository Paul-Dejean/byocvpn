use std::{collections::HashMap, sync::Mutex};

use byocvpn_core::cloud_provider::{SpawnJob, SpawnStepStatus};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSpawnJob {
    #[serde(flatten)]
    pub job: SpawnJob,
    pub instance_id: Option<String>,
    pub step_statuses: HashMap<String, SpawnStepStatus>,
}

pub struct SpawnJobRegistry(Mutex<HashMap<String, ActiveSpawnJob>>);

impl SpawnJobRegistry {
    pub fn new() -> Self {
        SpawnJobRegistry(Mutex::new(HashMap::new()))
    }

    pub fn register(&self, job: SpawnJob) {
        if let Ok(mut registry) = self.0.lock() {
            registry.insert(
                job.job_id.clone(),
                ActiveSpawnJob {
                    job,
                    instance_id: None,
                    step_statuses: HashMap::new(),
                },
            );
        }
    }

    pub fn update_step_status(&self, job_id: &str, step_id: &str, status: SpawnStepStatus) {
        if let Ok(mut registry) = self.0.lock() {
            if let Some(entry) = registry.get_mut(job_id) {
                entry.step_statuses.insert(step_id.to_string(), status);
            }
        }
    }

    pub fn set_instance_id(&self, job_id: &str, instance_id: String) {
        if let Ok(mut registry) = self.0.lock() {
            if let Some(entry) = registry.get_mut(job_id) {
                entry.instance_id = Some(instance_id);
            }
        }
    }

    pub fn instance_ids_in_progress(&self) -> Vec<String> {
        self.0
            .lock()
            .map(|registry| {
                registry
                    .values()
                    .filter_map(|entry| entry.instance_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn deregister(&self, job_id: &str) {
        if let Ok(mut registry) = self.0.lock() {
            registry.remove(job_id);
        }
    }

    pub fn list(&self) -> Vec<ActiveSpawnJob> {
        self.0
            .lock()
            .map(|registry| registry.values().cloned().collect())
            .unwrap_or_default()
    }
}
