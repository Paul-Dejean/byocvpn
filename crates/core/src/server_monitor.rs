use std::{
    collections::HashMap,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use jiff::{SignedDuration, Timestamp};
use tracing::{debug, info, warn};

use crate::{
    app_context::AppContext,
    daemon_client::{DaemonClient, fetch_vpn_status},
    instances::terminate_instance,
    ledger::LedgerEntry,
};

const MIN_AUTO_TERMINATE_MINUTES: u64 = 5;

pub trait ServerMonitorObserver: Send + Sync {
    fn on_long_running(&self, entry: &LedgerEntry, running_for: Duration);
    fn on_auto_terminated(&self, entry: &LedgerEntry, notifications_enabled: bool);
}

pub async fn run_monitor_tick(
    context: &AppContext,
    daemon_client: &dyn DaemonClient,
    observer: &dyn ServerMonitorObserver,
) {
    let running_entries = context.ledger.running_entries();
    if running_entries.is_empty() {
        return;
    }

    let notification_settings = context.settings.load_notification_settings();
    if notification_settings.notification_enabled {
        let threshold =
            Duration::from_secs(notification_settings.notification_threshold_minutes * 60);
        for (entry, running_for) in
            context
                .uptime_notices
                .select_due(&running_entries, threshold, Timestamp::now())
        {
            observer.on_long_running(&entry, running_for);
        }
    }

    run_auto_terminate_check(context, daemon_client, observer).await;
}

pub async fn run_auto_terminate_check(
    context: &AppContext,
    daemon_client: &dyn DaemonClient,
    observer: &dyn ServerMonitorObserver,
) {
    let running_entries = context.ledger.running_entries();
    if running_entries.is_empty() {
        return;
    }

    let connected_instance_id = match fetch_connection_state(daemon_client).await {
        ConnectionState::ConnectedTo(instance_id) => Some(instance_id),
        ConnectionState::Disconnected => None,
        ConnectionState::Unknown => return,
    };
    let now = Timestamp::now();
    let running_entries = refresh_idle_state(
        context,
        running_entries,
        connected_instance_id.as_deref(),
        now,
    );

    let auto_terminate_settings = context.settings.load_auto_terminate_settings();
    if !auto_terminate_settings.auto_terminate_enabled
        || auto_terminate_settings.auto_terminate_threshold_minutes == 0
    {
        return;
    }
    let threshold_minutes = auto_terminate_settings
        .auto_terminate_threshold_minutes
        .max(MIN_AUTO_TERMINATE_MINUTES);
    let threshold =
        i64::try_from(threshold_minutes).map_or(SignedDuration::MAX, SignedDuration::from_mins);

    let notifications_enabled = context
        .settings
        .load_notification_settings()
        .notification_enabled;

    for entry in select_idle_expired(
        &running_entries,
        connected_instance_id.as_deref(),
        threshold,
        now,
    ) {
        info!(instance_id = %entry.instance_id, provider = %entry.provider, threshold_minutes, "terminating idle server");
        match terminate_instance(context, entry.provider, &entry.region, &entry.instance_id).await {
            Ok(()) => observer.on_auto_terminated(entry, notifications_enabled),
            Err(error) => {
                warn!(instance_id = %entry.instance_id, error = %error.describe(), "failed to terminate idle server");
            }
        }
    }
}

pub fn select_idle_expired<'a>(
    entries: &'a [LedgerEntry],
    connected_instance_id: Option<&str>,
    threshold: SignedDuration,
    now: Timestamp,
) -> Vec<&'a LedgerEntry> {
    entries
        .iter()
        .filter(|entry| connected_instance_id != Some(entry.instance_id.as_str()))
        .filter(|entry| {
            entry
                .idle_since
                .is_some_and(|idle_since| now.duration_since(idle_since) >= threshold)
        })
        .collect()
}

#[derive(Default, Debug)]
pub struct UptimeNoticeTracker {
    last_notified_at: Mutex<HashMap<String, Timestamp>>,
}

impl UptimeNoticeTracker {
    pub fn select_due(
        &self,
        entries: &[LedgerEntry],
        threshold: Duration,
        now: Timestamp,
    ) -> Vec<(LedgerEntry, Duration)> {
        let mut last_notified_at = self
            .last_notified_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        entries
            .iter()
            .filter_map(|entry| {
                let running_for = elapsed_between(entry.launched_at, now);
                if running_for < threshold {
                    return None;
                }
                let is_due = last_notified_at
                    .get(&entry.instance_id)
                    .is_none_or(|last| elapsed_between(*last, now) >= threshold);
                if !is_due {
                    return None;
                }
                last_notified_at.insert(entry.instance_id.clone(), now);
                Some((entry.clone(), running_for))
            })
            .collect()
    }
}

enum ConnectionState {
    Unknown,
    Disconnected,
    ConnectedTo(String),
}

async fn fetch_connection_state(daemon_client: &dyn DaemonClient) -> ConnectionState {
    let vpn_status = match fetch_vpn_status(daemon_client).await {
        Ok(status) => status,
        Err(error) => {
            warn!(%error, "skipping this round — could not read VPN status, refusing to terminate a possibly-connected server"
            );
            return ConnectionState::Unknown;
        }
    };

    debug!(
        "raw daemon VpnStatus: connected={}, instance={:?}, connected_at={:?}",
        vpn_status.connected, vpn_status.instance, vpn_status.connected_at
    );

    if !vpn_status.connected {
        return ConnectionState::Disconnected;
    }
    if let Some(connected_instance) = vpn_status.instance {
        return ConnectionState::ConnectedTo(connected_instance.instance_id);
    }
    warn!("skipping this round — VPN reports connected but no instance is identified");
    ConnectionState::Unknown
}

fn refresh_idle_state(
    context: &AppContext,
    entries: Vec<LedgerEntry>,
    connected_instance_id: Option<&str>,
    now: Timestamp,
) -> Vec<LedgerEntry> {
    entries
        .into_iter()
        .map(|mut entry| {
            let is_connected = connected_instance_id == Some(entry.instance_id.as_str());
            if is_connected && entry.idle_since.is_some() {
                context.ledger.mark_connected(&entry.instance_id);
                entry.mark_connected();
            } else if !is_connected && entry.idle_since.is_none() {
                context.ledger.mark_idle_since(&entry.instance_id, now);
                entry.mark_idle_since(now);
            }
            entry
        })
        .collect()
}

fn elapsed_between(earlier: Timestamp, later: Timestamp) -> Duration {
    Duration::try_from(later.duration_since(earlier)).unwrap_or_default()
}
