use std::{future::Future, time::Duration};

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
    let spinner = start_spinner(message);
    let outcome = operation.await;
    finish_step(&spinner, message, outcome.is_ok());
    outcome
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

fn finish_step(spinner: &ProgressBar, message: &str, succeeded: bool) {
    spinner.set_style(
        ProgressStyle::with_template("{msg}").unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    let symbol = if succeeded { "✔" } else { "✘" };
    spinner.finish_with_message(format!("{} {}", symbol, message));
}
