use std::{future::Future, sync::Mutex, time::Duration};

use byocvpn_core::{
    cloud_provider::{CloudProviderName, InstanceInfo, JobStep, JobStepStatus},
    error::{Error, ErrorPayload},
    job_registry::SpawnJobState,
    jobs::{
        JobObserver, enable_region::EnableRegionObserver,
        provision_account::ProvisionAccountObserver, spawn::SpawnJobObserver,
    },
};
use indicatif::{ProgressBar, ProgressStyle};

const SPINNER_TICK_INTERVAL: Duration = Duration::from_millis(80);

pub async fn with_spinner<ResultValue, ErrorValue>(
    message: &str,
    operation: impl Future<Output = Result<ResultValue, ErrorValue>>,
) -> Result<ResultValue, ErrorValue> {
    let spinner = start_spinner(message);
    let outcome = operation.await;
    spinner.finish_and_clear();
    outcome
}

pub async fn with_step<ResultValue, ErrorValue>(
    message: &str,
    operation: impl Future<Output = Result<ResultValue, ErrorValue>>,
) -> Result<ResultValue, ErrorValue> {
    let step_spinner = StepSpinner::start(message);
    let outcome = operation.await;
    step_spinner.finish(outcome.is_ok());
    outcome
}

#[derive(Default)]
pub struct TerminalJobObserver {
    active_step_spinner: Mutex<Option<StepSpinner>>,
}

impl JobObserver for TerminalJobObserver {
    fn on_step_changed(&self, step: &JobStep, status: JobStepStatus, _error: Option<ErrorPayload>) {
        match status {
            JobStepStatus::Running => self.start_step(&step.label),
            JobStepStatus::Completed => self.finish_step(true),
            JobStepStatus::Failed => self.finish_step(false),
            JobStepStatus::Pending => {}
        }
    }
}

impl SpawnJobObserver for TerminalJobObserver {
    fn on_job_started(&self, _job: &SpawnJobState) {}

    fn on_instance_launched(&self, _instance: &InstanceInfo) {}

    fn on_job_completed(&self, _instance: &InstanceInfo) {}

    fn on_job_failed(&self, _error: &Error) {}
}

impl ProvisionAccountObserver for TerminalJobObserver {
    fn on_job_started(&self, _job_id: &str, _steps: &[JobStep]) {}

    fn on_job_completed(&self, _provider: &CloudProviderName) {}

    fn on_job_failed(&self, _error: &Error) {}
}

impl EnableRegionObserver for TerminalJobObserver {
    fn on_job_started(&self, _job_id: &str, _steps: &[JobStep]) {}

    fn on_job_completed(&self, _region: &str) {}

    fn on_job_failed(&self, _error: &Error) {}
}

impl TerminalJobObserver {
    fn start_step(&self, label: &str) {
        if let Ok(mut active_step_spinner) = self.active_step_spinner.lock() {
            *active_step_spinner = Some(StepSpinner::start(label));
        }
    }

    fn finish_step(&self, succeeded: bool) {
        let finished_step_spinner = self
            .active_step_spinner
            .lock()
            .ok()
            .and_then(|mut active_step_spinner| active_step_spinner.take());
        if let Some(step_spinner) = finished_step_spinner {
            step_spinner.finish(succeeded);
        }
    }
}

struct StepSpinner {
    spinner: ProgressBar,
    message: String,
}

impl StepSpinner {
    fn start(message: &str) -> Self {
        Self {
            spinner: start_spinner(message),
            message: message.to_string(),
        }
    }

    fn finish(self, succeeded: bool) {
        self.spinner.set_style(
            ProgressStyle::with_template("{msg}").unwrap_or_else(|_| ProgressStyle::default_bar()),
        );
        let symbol = if succeeded { "✔" } else { "✘" };
        self.spinner
            .finish_with_message(format!("{symbol} {}", self.message));
    }
}

fn start_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(SPINNER_TICK_INTERVAL);
    spinner
}
