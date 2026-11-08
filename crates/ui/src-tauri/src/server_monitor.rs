use std::time::Duration;

use byocvpn_core::{
    app_context::AppContext,
    ledger::LedgerEntry,
    server_monitor::{self, ServerMonitorObserver},
};
use humantime::format_duration;
use log::warn;
use tauri::{AppHandle, Manager, async_runtime};
use tauri_plugin_notification::NotificationExt;
use tauri_specta::Event;
use tokio::time::interval;

use crate::events::InstanceAutoTerminatedEvent;

const CHECK_INTERVAL_SECONDS: u64 = 60;

pub fn start_server_monitor(app_handle: AppHandle) {
    async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECONDS));
        loop {
            ticker.tick().await;
            let context = app_handle.state::<AppContext>();
            let observer = DesktopMonitorObserver {
                app_handle: app_handle.clone(),
            };
            server_monitor::run_monitor_tick(&context, context.daemon_client.as_ref(), &observer)
                .await;
        }
    });
}

pub async fn run_auto_terminate_check(app_handle: &AppHandle) {
    let context = app_handle.state::<AppContext>();
    let observer = DesktopMonitorObserver {
        app_handle: app_handle.clone(),
    };
    server_monitor::run_auto_terminate_check(&context, context.daemon_client.as_ref(), &observer)
        .await;
}

struct DesktopMonitorObserver {
    app_handle: AppHandle,
}

impl ServerMonitorObserver for DesktopMonitorObserver {
    fn on_long_running(&self, entry: &LedgerEntry, running_for: Duration) {
        let body = format!(
            "Your {} server in {} has been running for {}. Consider terminating it to avoid unnecessary costs.",
            entry.provider.to_string().to_uppercase(),
            entry.region,
            format_duration(running_for)
        );
        self.show_notification("ByocVPN — Server Running", &body);
    }

    fn on_auto_terminated(&self, entry: &LedgerEntry, notifications_enabled: bool) {
        if notifications_enabled {
            let body = format!(
                "Your {} server in {} was automatically terminated after running idle without a VPN connection.",
                entry.provider.to_string().to_uppercase(),
                entry.region
            );
            self.show_notification("ByocVPN — Server Auto-Terminated", &body);
        }
        if let Err(error) = (InstanceAutoTerminatedEvent {
            instance_id: entry.instance_id.clone(),
        })
        .emit(&self.app_handle)
        {
            warn!("[auto-terminate] failed to emit event: {error}");
        }
    }
}

impl DesktopMonitorObserver {
    fn show_notification(&self, title: &str, body: &str) {
        if let Err(error) = self
            .app_handle
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            warn!("Failed to show notification {title:?}: {error}");
        }
    }
}
