use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, JobStep, JobStepStatus},
    error::Error,
    job_registry::SpawnJobState,
    jobs::{
        JobObserver, enable_region::EnableRegionObserver,
        provision_account::ProvisionAccountObserver, spawn::SpawnJobObserver,
    },
};
use log::warn;
use serde::Serialize;
use tauri::{
    AppHandle, Manager,
    ipc::{Channel, IpcResponse},
};
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpawnInstanceEvent {
    Started {
        job: SpawnJobState,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: JobStepStatus,
        error: Option<String>,
    },
    InstanceLaunched {
        instance: InstanceInfo,
    },
    Complete {
        instance: InstanceInfo,
    },
    Failed {
        error: String,
    },
}

#[derive(Clone, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvisionAccountEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        job_id: String,
        steps: Vec<JobStep>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: JobStepStatus,
        error: Option<String>,
    },
    Complete {
        provider: CloudProviderName,
    },
    Failed {
        error: String,
    },
}

#[derive(Clone, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnableRegionEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        job_id: String,
        steps: Vec<JobStep>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        step_id: String,
        status: JobStepStatus,
        error: Option<String>,
    },
    Complete {
        region: String,
    },
    Failed {
        error: String,
    },
}

pub struct SpawnEventForwarder {
    app_handle: AppHandle,
    channel: Channel<SpawnInstanceEvent>,
}

impl SpawnEventForwarder {
    pub const fn new(app_handle: AppHandle, channel: Channel<SpawnInstanceEvent>) -> Self {
        Self {
            app_handle,
            channel,
        }
    }
}

impl JobObserver for SpawnEventForwarder {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, error: Option<String>) {
        send_channel_event(
            &self.channel,
            SpawnInstanceEvent::Progress {
                step_id: step.id.clone(),
                status,
                error,
            },
        );
    }
}

impl SpawnJobObserver for SpawnEventForwarder {
    fn on_job_started(&self, job: &SpawnJobState) {
        send_channel_event(
            &self.channel,
            SpawnInstanceEvent::Started { job: job.clone() },
        );
    }

    fn on_instance_launched(&self, instance: &InstanceInfo) {
        send_channel_event(
            &self.channel,
            SpawnInstanceEvent::InstanceLaunched {
                instance: instance.clone(),
            },
        );
    }

    fn on_job_completed(&self, instance: &InstanceInfo) {
        notify_server_deployed(&self.app_handle, instance);
        send_channel_event(
            &self.channel,
            SpawnInstanceEvent::Complete {
                instance: instance.clone(),
            },
        );
    }

    fn on_job_failed(&self, error: &Error) {
        send_channel_event(
            &self.channel,
            SpawnInstanceEvent::Failed {
                error: error.to_string(),
            },
        );
    }
}

pub struct ProvisionAccountEventForwarder {
    channel: Channel<ProvisionAccountEvent>,
}

impl ProvisionAccountEventForwarder {
    pub const fn new(channel: Channel<ProvisionAccountEvent>) -> Self {
        Self { channel }
    }
}

impl JobObserver for ProvisionAccountEventForwarder {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, error: Option<String>) {
        send_channel_event(
            &self.channel,
            ProvisionAccountEvent::Progress {
                step_id: step.id.clone(),
                status,
                error,
            },
        );
    }
}

impl ProvisionAccountObserver for ProvisionAccountEventForwarder {
    fn on_job_started(&self, job_id: &str, steps: &[JobStep]) {
        send_channel_event(
            &self.channel,
            ProvisionAccountEvent::Started {
                job_id: job_id.to_string(),
                steps: steps.to_vec(),
            },
        );
    }

    fn on_job_completed(&self, provider: &CloudProviderName) {
        send_channel_event(
            &self.channel,
            ProvisionAccountEvent::Complete {
                provider: provider.clone(),
            },
        );
    }

    fn on_job_failed(&self, error: &Error) {
        send_channel_event(
            &self.channel,
            ProvisionAccountEvent::Failed {
                error: error.to_string(),
            },
        );
    }
}

pub struct EnableRegionEventForwarder {
    channel: Channel<EnableRegionEvent>,
}

impl EnableRegionEventForwarder {
    pub const fn new(channel: Channel<EnableRegionEvent>) -> Self {
        Self { channel }
    }
}

impl JobObserver for EnableRegionEventForwarder {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, error: Option<String>) {
        send_channel_event(
            &self.channel,
            EnableRegionEvent::Progress {
                step_id: step.id.clone(),
                status,
                error,
            },
        );
    }
}

impl EnableRegionObserver for EnableRegionEventForwarder {
    fn on_job_started(&self, job_id: &str, steps: &[JobStep]) {
        send_channel_event(
            &self.channel,
            EnableRegionEvent::Started {
                job_id: job_id.to_string(),
                steps: steps.to_vec(),
            },
        );
    }

    fn on_job_completed(&self, region: &str) {
        send_channel_event(
            &self.channel,
            EnableRegionEvent::Complete {
                region: region.to_string(),
            },
        );
    }

    fn on_job_failed(&self, error: &Error) {
        send_channel_event(
            &self.channel,
            EnableRegionEvent::Failed {
                error: error.to_string(),
            },
        );
    }
}

fn send_channel_event<Event: IpcResponse>(channel: &Channel<Event>, event: Event) {
    if let Err(error) = channel.send(event) {
        warn!("Failed to send job event to the frontend: {error}");
    }
}

fn notify_server_deployed(app_handle: &AppHandle, instance: &InstanceInfo) {
    if has_focused_window(app_handle) {
        return;
    }

    let body = format!(
        "Your {} server in {} is ready to connect.",
        instance.provider.to_string().to_uppercase(),
        instance.region
    );
    if let Err(error) = app_handle
        .notification()
        .builder()
        .title("ByocVPN — Server Ready")
        .body(&body)
        .show()
    {
        warn!("Failed to send server-deployed notification: {error}");
    }
}

fn has_focused_window(app_handle: &AppHandle) -> bool {
    app_handle.webview_windows().values().any(|window| {
        window.is_visible().unwrap_or(false)
            && window.is_focused().unwrap_or(false)
            && !window.is_minimized().unwrap_or(false)
    })
}
