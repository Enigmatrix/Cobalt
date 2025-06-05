use std::sync::Arc;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{
    AlertEvent, App, Duration, Reason, Ref, ReminderEvent, Target, Timestamp, TriggerAction,
};
use platform::objects::{Process, Progress, Timestamp as PlatformTimestamp, ToastManager, Window};
use util::error::Result;
use util::future::sync::Mutex;
use util::time::ToTicks;
use util::tracing::{debug, info, ResultTraceExt};

use crate::cache::{Cache, KillableProcessId};

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
        // Retain alive entries in cache before we start processing
        // - avoids dimming dead windows / killing dead processes
        {
            let mut cache = self.cache.lock().await;
            cache.retain_cache()?;
        }
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
                triggered_reminder.usage_limit,
            )
            .warn();
            self.mgr
                .insert_reminder_event(&ReminderEvent {
                    id: Default::default(),
                    reminder_id: triggered_reminder.reminder.id.clone(),
                    timestamp: now.to_ticks(),
                    reason: Reason::Hit,
                })
                .await?;
        }

        Ok(())
    }

    /// Get the [ProcessId]s for the given [Target].
    pub async fn processes_for_target(
        &mut self,
        target: &Target,
    ) -> Result<Vec<(Ref<App>, KillableProcessId)>> {
        let cache = self.cache.lock().await;
        let processes = self
            .mgr
            .target_apps(target)
            .await?
            .iter()
            .flat_map(move |app| {
                cache
                    .processes_for_app(app)
                    .into_iter()
                    .map(|pid| (app.clone(), pid))
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
                for (app, pid) in processes {
                    info!(?alert, "killing process {:?} for app {:?}", pid, app);
                    match pid {
                        KillableProcessId::Win32(pid) => {
                            let process = Process::new_killable(pid)?;
                            self.handle_kill_action(&process).warn();

                            let mut cache = self.cache.lock().await;
                            cache.remove_process(pid);
                        }
                        KillableProcessId::Aumid(aumid) => {
                            Process::kill_uwp(&aumid).await.warn();

                            let mut cache = self.cache.lock().await;
                            cache.remove_app(app);
                        }
                    }
                }
            }
            TriggerAction::Dim { duration } => {
                let windows = self.windows_for_target(&alert.target).await?;
                let start = timestamp.unwrap_or(now.to_ticks());
                let end: Timestamp = now.to_ticks();
                let progress = (end - start) as f64 / (*duration as f64);
                for window in windows {
                    let dim_level = 1.0f64 - progress.min(1.0f64);
                    if dim_level == 1.0f64 {
                        info!(?alert, "start dimming window {:?}", window);
                    } else if dim_level == 0.0f64 {
                        info!(?alert, "end dimming window {:?}", window);
                    } else {
                        debug!(?alert, "dimming window {:?} to {}", window, dim_level);
                    }
                    self.handle_dim_action(&window, dim_level).warn();
                }
            }
            TriggerAction::Message { content } => {
                if timestamp.is_none() {
                    info!(?alert, "send message {:?}: {:?}", name, content);
                    self.handle_message_action(name, content).warn();
                }
            }
        }
        if timestamp.is_none() {
            self.mgr
                .insert_alert_event(&AlertEvent {
                    id: Default::default(),
                    alert_id: alert.id.clone(),
                    timestamp: now.to_ticks(),
                    reason: Reason::Hit,
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
        ToastManager::show_alert(&format!("Alert: {}", name), msg)?;
        Ok(())
    }

    /// Dim the [Window] to the given opacity.
    pub fn handle_dim_action(&self, window: &Window, dim: f64) -> Result<()> {
        window.dim(dim)?;
        Ok(())
    }

    /// Message the user with the given [TriggeredReminder]'s name and message and progress.
    pub fn handle_reminder_message(
        &self,
        name: &str,
        msg: &str,
        value: f64,
        usage_limit: Duration,
    ) -> Result<()> {
        ToastManager::show_reminder(
            name,
            msg,
            Progress {
                title: None,
                value,
                value_string_override: None,
                status: format!(
                    "{} / {}",
                    util::time::human_duration(
                        platform::objects::Duration::from_ticks(
                            (usage_limit as f64 * value) as i64
                        )
                        .into()
                    ),
                    util::time::human_duration(
                        platform::objects::Duration::from_ticks(usage_limit).into()
                    )
                ),
            },
        )?;
        Ok(())
    }
}
