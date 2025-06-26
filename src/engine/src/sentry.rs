use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{
    AlertEvent, App, Duration, Reason, Ref, ReminderEvent, Target, Timestamp, TriggerAction,
};
use platform::events::TabChange;
use platform::objects::{Process, Progress, Timestamp as PlatformTimestamp, ToastManager, Window};
use platform::web::{BaseWebsiteUrl, BrowserDetector, WebsiteInfo};
use util::error::Result;
use util::future::sync::Mutex;
use util::time::ToTicks;
use util::tracing::{ResultTraceExt, debug, info};

use crate::cache::{Cache, KillableProcessId};

/// Watcher to track [TriggeredAlert]s and [TriggeredReminder]s and take action on them.
pub struct Sentry {
    cache: Arc<Mutex<Cache>>,
    mgr: AlertManager,
    // Invariant: the dimmed_websites, killed_websites map is *wholly* updated. That is, run() executes
    // and updates this after clearing it with *all* alerts' websites.
    dimmed_websites: HashMap<String, f64>,
    killed_websites: HashMap<String, ()>, // TODO: add stuff
}

const MIN_DIM_LEVEL: f64 = 0.5;

impl Sentry {
    /// Create a new [Sentry] with the given [Cache] and [Database].
    pub fn new(cache: Arc<Mutex<Cache>>, db: Database) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self {
            cache,
            mgr,
            dimmed_websites: HashMap::new(),
            killed_websites: HashMap::new(),
        })
    }

    /// Dim the browser windows matching the given [TabUpdate].
    pub async fn handle_tab_change(&mut self, tab_change: TabChange) -> Result<()> {
        let browser_detect = BrowserDetector::new()?;

        let changed_browser_windows = match tab_change {
            TabChange::Tab { window } => HashSet::from_iter(vec![window]),
            TabChange::Tick => {
                let cache = self.cache.lock().await;
                cache.browser_windows().await
            }
        };

        for window in changed_browser_windows {
            let browser_url = browser_detect.chromium_url(&window).unwrap_or_default();
            let url = browser_url.url.unwrap_or_default();
            let url = WebsiteInfo::url_to_base_url(&url)?.to_string();

            if self.killed_websites.contains_key(&url) {
                browser_detect.close_current_tab(&window).warn();
            } else if let Some(dim_level) = self.dimmed_websites.get(&url) {
                self.handle_dim_action(&window, *dim_level).warn();
            } else {
                // no action taken, so we dim to back to full
                self.handle_dim_action(&window, 1.0f64).warn();
            }
        }

        Ok(())
    }

    /// Run the [Sentry] to check for triggered alerts and reminders and take action on them.
    pub async fn run(&mut self, now: PlatformTimestamp) -> Result<()> {
        // Retain alive entries in cache before we start processing
        // - avoids dimming dead windows / killing dead processes
        {
            let mut cache = self.cache.lock().await;
            cache.retain_cache().await?;
        }

        self.dimmed_websites.clear();
        self.killed_websites.clear();

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

    /// Get the apps and processes for the given [Target], those that are platform-based, rather than websites.
    pub async fn processes_and_websites_for_target(
        &mut self,
        target: &Target,
    ) -> Result<(Vec<(Ref<App>, KillableProcessId)>, Vec<BaseWebsiteUrl>)> {
        let target_apps = self.mgr.target_apps(target).await?;

        let cache = self.cache.lock().await;
        let processes = target_apps
            .iter()
            .flat_map(move |app| {
                cache
                    .platform_processes_for_app(app)
                    .into_iter()
                    .map(|pid| (app.clone(), pid))
            })
            .collect();

        // yes, we lock twice here ...
        let cache = self.cache.lock().await;
        let websites = target_apps
            .iter()
            .flat_map(move |app| cache.websites_for_app(app).cloned().collect::<Vec<_>>())
            .collect();
        Ok((processes, websites))
    }

    /// Get the [Window]s and websites for the given [Target].
    pub async fn windows_and_websites_for_target(
        &mut self,
        target: &Target,
    ) -> Result<(Vec<Window>, Vec<BaseWebsiteUrl>)> {
        let target_apps = self.mgr.target_apps(target).await?;

        let cache = self.cache.lock().await;
        let windows = target_apps
            .iter()
            .flat_map(move |app| {
                cache
                    .platform_windows_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>() // ew
            })
            .collect();

        // yes, we lock twice here ...
        let cache = self.cache.lock().await;
        let websites = target_apps
            .iter()
            .flat_map(move |app| cache.websites_for_app(app).cloned().collect::<Vec<_>>())
            .collect();
        Ok((windows, websites))
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
                let (processes, websites) = self
                    .processes_and_websites_for_target(&alert.target)
                    .await?;

                self.killed_websites
                    .extend(websites.into_iter().map(|url| (url.to_string(), ())));

                for (app, pid) in processes {
                    info!(?alert, "killing process {:?} for app {:?}", pid, app);
                    match pid {
                        KillableProcessId::Win32(pid) => {
                            let process = Process::new_killable(pid)?;
                            self.handle_kill_action(&process).warn();

                            let mut cache = self.cache.lock().await;
                            cache.remove_process(pid).await;
                        }
                        KillableProcessId::Aumid(aumid) => {
                            Process::kill_uwp(&aumid).await.warn();

                            let mut cache = self.cache.lock().await;
                            cache.remove_app(app).await;
                        }
                    }
                }
            }
            TriggerAction::Dim { duration } => {
                let (windows, websites) =
                    self.windows_and_websites_for_target(&alert.target).await?;
                let start = timestamp.unwrap_or(now.to_ticks());
                let end: Timestamp = now.to_ticks();
                let progress = (end - start) as f64 / (*duration as f64);
                let dim_level = 1.0f64 - (progress.min(1.0f64) * (1.0f64 - MIN_DIM_LEVEL));

                self.dimmed_websites
                    .extend(websites.into_iter().map(|url| (url.to_string(), dim_level)));

                for window in windows {
                    if dim_level == 1.0f64 {
                        info!(?alert, "start dimming window {:?}", window);
                    } else if dim_level == MIN_DIM_LEVEL && progress <= 1.01f64 {
                        // allow for floating point imprecision. check if we reach ~100% progress
                        info!(?alert, "max dim window reached for {:?}", window);
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
        ToastManager::show_alert(&format!("Alert: {name}"), msg)?;
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
