#[cfg(windows)]
use std::process::ExitCode;

#[cfg(not(windows))]
use byocvpn_core::error::Result;
use byocvpn_daemon::daemon::run_daemon;
#[cfg(windows)]
use tokio::runtime::Builder as RuntimeBuilder;

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> Result<()> {
    run_daemon().await
}

#[cfg(windows)]
fn main() -> ExitCode {
    if windows_service_impl::start_as_service().is_ok() {
        return ExitCode::SUCCESS;
    }
    run_daemon_in_foreground()
}

#[cfg(windows)]
#[expect(
    clippy::print_stderr,
    reason = "the logger is only initialized once the daemon runs, so startup failures can only reach stderr"
)]
fn run_daemon_in_foreground() -> ExitCode {
    let runtime = match RuntimeBuilder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Failed to build tokio runtime: {error}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(async {
        match run_daemon().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Daemon error: {error}");
                ExitCode::FAILURE
            }
        }
    })
}

#[cfg(windows)]
mod windows_service_impl {
    use std::{ffi::OsString, sync::mpsc, time::Duration};

    use byocvpn_daemon::daemon::run_daemon;
    use log::{debug, error, info, warn};
    use tokio::{runtime::Builder as RuntimeBuilder, task::spawn_blocking};
    use windows_service::{
        Result as ServiceResult, define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    define_windows_service!(ffi_service_main, service_main);

    pub fn start_as_service() -> ServiceResult<()> {
        service_dispatcher::start(get_service_name(), ffi_service_main)
    }

    const fn get_service_name() -> &'static str {
        if cfg!(debug_assertions) {
            "byocvpn-daemon-dev"
        } else {
            "byocvpn-daemon"
        }
    }

    fn service_main(_arguments: Vec<OsString>) {
        let (shutdown_sender, shutdown_receiver) = mpsc::channel::<()>();

        let status_handle =
            match service_control_handler::register(get_service_name(), move |control_event| {
                match control_event {
                    ServiceControl::Stop | ServiceControl::Shutdown => {
                        if shutdown_sender.send(()).is_err() {
                            debug!("Service stop requested after the daemon already stopped");
                        }
                        ServiceControlHandlerResult::NoError
                    }
                    ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                    _ => ServiceControlHandlerResult::NotImplemented,
                }
            }) {
                Ok(status_handle) => status_handle,
                Err(error) => {
                    error!("Failed to register service control handler: {error}");
                    return;
                }
            };

        if let Err(error) = status_handle.set_service_status(build_service_status(
            ServiceState::Running,
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        )) {
            error!("Failed to set service status to Running: {error}");
            return;
        }

        let runtime = match RuntimeBuilder::new_multi_thread().enable_all().build() {
            Ok(runtime) => runtime,
            Err(error) => {
                error!("Failed to build tokio runtime: {error}");
                return;
            }
        };

        runtime.block_on(async {
            tokio::select! {
                result = run_daemon() => {
                    if let Err(error) = result {
                        error!("Daemon exited with error: {error}");
                    }
                }
                _ = spawn_blocking(move || shutdown_receiver.recv()) => {
                    info!("Service stop signal received");
                }
            }
        });

        if let Err(error) = status_handle.set_service_status(build_service_status(
            ServiceState::Stopped,
            ServiceControlAccept::empty(),
        )) {
            warn!("Failed to set service status to Stopped: {error}");
        }
    }

    fn build_service_status(
        current_state: ServiceState,
        controls_accepted: ServiceControlAccept,
    ) -> ServiceStatus {
        ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state,
            controls_accepted,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        }
    }
}
