use byocvpn_daemon::daemon::run_daemon;

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> Result<()> {
    run_daemon().await
}

#[cfg(windows)]
fn main() {
    if windows_service_impl::start_as_service().is_err() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(async {
                if let Err(error) = run_daemon().await {
                    eprintln!("Daemon error: {}", error);
                    std::process::exit(1);
                }
            });
    }
}

#[cfg(windows)]
mod windows_service_impl {
    use std::sync::mpsc;
    use std::time::Duration;

    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    define_windows_service!(ffi_service_main, service_main);

    pub fn start_as_service() -> windows_service::Result<()> {
        let service_name = if cfg!(debug_assertions) {
            "byocvpn-daemon-dev"
        } else {
            "byocvpn-daemon"
        };
        service_dispatcher::start(service_name, ffi_service_main)
    }

    fn service_main(_arguments: Vec<std::ffi::OsString>) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let status_handle = service_control_handler::register(
            if cfg!(debug_assertions) {
                "byocvpn-daemon-dev"
            } else {
                "byocvpn-daemon"
            },
            move |control_event| match control_event {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    let _ = shutdown_tx.send(());
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            },
        )
        .expect("failed to register service control handler");

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .expect("failed to set service status to Running");

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        runtime.block_on(async {
            tokio::select! {
                result = byocvpn_daemon::daemon::run_daemon() => {
                    if let Err(error) = result {
                        log::error!("Daemon exited with error: {}", error);
                    }
                }
                _ = tokio::task::spawn_blocking(move || shutdown_rx.recv()) => {
                    log::info!("Service stop signal received");
                }
            }
        });

        if let Err(error) = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        }) {
            log::warn!("Failed to set service status to Stopped: {}", error);
        }
    }
}
