use std::sync::Arc;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{AlertEvent, ReminderEvent, Target, Timestamp, TriggerAction};
use platform::objects::{
    Process, ProcessId, Progress, Timestamp as PlatformTimestamp, ToastManager, Window,
};
use util::error::Result;
use util::future::sync::Mutex;
use util::time::ToTicks;
use util::tracing::{info, ResultTraceExt};

use crate::cache::Cache;

/// Watcher to track [TriggeredAlert]s and [TriggeredReminder]s and take action on them.
pub struct Sentry {
    cache: Arc<Mutex<Cache>>,
    mgr: AlertManager,
}

impl Sentry {
    /// Create a new [Sentry] with the given [Cache] and [Database].
    pub fn new(cache: Arc<Mutex<Cache>>, db: Database) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self { cache, mgr })
    }

    /// Run the [Sentry] to check for triggered alerts and reminders and take action on them.
    pub async fn run(&mut self, now: PlatformTimestamp) -> Result<()> {
        let alerts_hits = self.mgr.triggered_alerts(&now).await?;
        for triggered_alert in alerts_hits {
            self.handle_alert(&triggered_alert, now).await?;
        }

        let reminder_hits = self.mgr.triggered_reminders(&now).await?;
        for triggered_reminder in reminder_hits {
            info!(?triggered_reminder, "send message");
            self.handle_reminder_message(
                &triggered_reminder.name,
                &triggered_reminder.reminder.message,
                triggered_reminder.reminder.threshold,
            )
            .warn();
            self.mgr
                .insert_reminder_event(&ReminderEvent {
                    id: Default::default(),
                    reminder: triggered_reminder.reminder.id.0.into(),
                    timestamp: now.to_ticks(),
                })
                .await?;
        }

        Ok(())
    }

    /// Get the [ProcessId]s for the given [Target].
    pub async fn processes_for_target(&mut self, target: &Target) -> Result<Vec<ProcessId>> {
        let cache = self.cache.lock().await;
        let processes = self
            .mgr
            .target_apps(target)
            .await?
            .iter()
            .flat_map(move |app| {
                cache.processes_for_app(app).cloned().collect::<Vec<_>>() // ew
            })
            .collect();
        Ok(processes)
    }

    /// Get the [Window]s for the given [Target].
    pub async fn windows_for_target(&mut self, target: &Target) -> Result<Vec<Window>> {
        let cache = self.cache.lock().await;
        let window = self
            .mgr
            .target_apps(target)
            .await?
            .iter()
            .flat_map(move |app| {
                cache.windows_for_app(app).cloned().collect::<Vec<_>>() // ew
            })
            .collect();
        Ok(window)
    }

    /// Handle the given [TriggeredAlert] and take action on it.
    pub async fn handle_alert(
        &mut self,
        triggered_alert: &TriggeredAlert,
        now: PlatformTimestamp,
    ) -> Result<()> {
        let TriggeredAlert {
            alert,
            timestamp,
            name,
        } = triggered_alert;
        match &alert.trigger_action {
            TriggerAction::Kill => {
                let processes = self.processes_for_target(&alert.target).await?;
                let mut cache = self.cache.lock().await;
                for pid in processes {
                    info!(?alert, "killing process {:?}", pid);
                    let process = Process::new_killable(pid)?;
                    self.handle_kill_action(&process).warn();
                    cache.remove_process(pid);
                }
            }
            TriggerAction::Dim(dur) => {
                let windows = self.windows_for_target(&alert.target).await?;
                let start = timestamp.unwrap_or(now.to_ticks());
                let end: Timestamp = now.to_ticks();
                let progress = (end - start) as f64 / (*dur as f64);
                for window in windows {
                    let dim_level = 1.0f64 - progress.min(1.0f64);
                    info!(?alert, "dimming window {:?} to {}", window, dim_level);
                    self.handle_dim_action(&window, dim_level).warn();
                }
            }
            TriggerAction::Message(msg) => {
                if timestamp.is_none() {
                    info!(?alert, "send message {:?}: {:?}", name, msg);
                    self.handle_message_action(name, msg).warn();
                }
            }
        }
        if timestamp.is_none() {
            self.mgr
                .insert_alert_event(&AlertEvent {
                    id: Default::default(),
                    alert: alert.id.0.clone().into(),
                    timestamp: now.to_ticks(),
                })
                .await?;
        }
        Ok(())
    }

    /// Kill the [Process]
    pub fn handle_kill_action(&self, process: &Process) -> Result<()> {
        process.kill(None)?;
        Ok(())
    }

    /// Message the user with the given [TriggeredAlert]'s name and message.
    pub fn handle_message_action(&self, name: &str, msg: &str) -> Result<()> {
        ToastManager::show_basic(name, msg)?;
        Ok(())
    }

    /// Dim the [Window] to the given opacity.
    pub fn handle_dim_action(&self, window: &Window, dim: f64) -> Result<()> {
        window.dim(dim)?;
        Ok(())
    }

    /// Message the user with the given [TriggeredReminder]'s name and message and progress.
    pub fn handle_reminder_message(&self, name: &str, msg: &str, value: f64) -> Result<()> {
        ToastManager::show_with_progress(
            name,
            msg,
            Progress {
                title: String::new(),
                value,
                value_string_override: "".to_string(),
                //status: "1h/1h 40m".to_string(),
                status: "".to_string(),
            },
        )?;
        Ok(())
    }
}
