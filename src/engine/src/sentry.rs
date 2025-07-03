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
    // Invariant: the website_actions map is *wholly* updated. That is, run() executes
    // and updates this after clearing it with *all* alerts' websites. This is easily
    // maintained since the two places its used - run() and handle_tab_change() are both &mut self.
    website_actions: HashMap<String, WebsiteAction>,
}

const MIN_DIM_LEVEL: f64 = 0.5;

#[derive(Clone, Debug)]
enum WebsiteAction {
    Dim(f64),
    Kill,
}

impl PartialEq<WebsiteAction> for WebsiteAction {
    fn eq(&self, other: &WebsiteAction) -> bool {
        match (self, other) {
            (WebsiteAction::Dim(dim_level), WebsiteAction::Dim(other_dim_level)) => {
                dim_level == other_dim_level
            }
            (WebsiteAction::Kill, WebsiteAction::Kill) => true,
            _ => false,
        }
    }
}

impl Eq for WebsiteAction {}

impl PartialOrd for WebsiteAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WebsiteAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (WebsiteAction::Dim(dim_level), WebsiteAction::Dim(other_dim_level)) => {
                if (dim_level - other_dim_level).abs() <= f64::EPSILON {
                    std::cmp::Ordering::Equal
                } else {
                    dim_level.total_cmp(other_dim_level)
                }
            }
            (WebsiteAction::Kill, WebsiteAction::Kill) => std::cmp::Ordering::Equal,
            (WebsiteAction::Kill, _) => std::cmp::Ordering::Greater,
            (_, WebsiteAction::Kill) => std::cmp::Ordering::Less,
        }
    }
}

impl Sentry {
    /// Create a new [Sentry] with the given [Cache] and [Database].
    pub fn new(cache: Arc<Mutex<Cache>>, db: Database) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self {
            cache,
            mgr,
            website_actions: HashMap::new(),
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
            // skip if window is dead or minimized
            if window.ptid().is_err() || window.is_minimized() {
                continue;
            }

            let element = browser_detect.get_chromium_element(&window)?;
            let url = browser_detect.chromium_url(&element).unwrap_or_default();
            let base_url = WebsiteInfo::url_to_base_url(&url.unwrap_or_default())?.to_string();

            if let Some(action) = self.website_actions.get(&base_url) {
                match action {
                    WebsiteAction::Dim(dim_level) => {
                        self.handle_dim_action(&window, *dim_level).warn();
                    }
                    WebsiteAction::Kill => {
                        browser_detect.close_current_tab(&element).warn();
                    }
                }
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

        // Reset the website actions - it will get repopulated by the alerts
        self.website_actions.clear();

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

                // set websites to kill - kill takes precedence over dim
                self.merge_website_actions(
                    websites
                        .into_iter()
                        .map(|url| (url.to_string(), WebsiteAction::Kill)),
                );

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

                // set websites to dim - kill takes precedence
                self.merge_website_actions(
                    websites
                        .into_iter()
                        .map(|url| (url.to_string(), WebsiteAction::Dim(dim_level))),
                );

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

    /// Get the apps + processes and website for the given [Target],
    async fn processes_and_websites_for_target(
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

    /// Get the [Window]s and websites for the given [Target],
    async fn windows_and_websites_for_target(
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

    fn merge_website_actions(&mut self, new: impl IntoIterator<Item = (String, WebsiteAction)>) {
        for (url, action) in new {
            let action = if let Some(existing) = self.website_actions.get(&url) {
                action.max(existing.clone())
            } else {
                action
            };
            self.website_actions.insert(url, action);
        }
    }
}
