use std::collections::HashMap;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{
    AlertEvent, App, Duration, Reason, Ref, ReminderEvent, Target, TriggerAction,
};
use platform::browser::{self, ArcBrowser, BrowserAction, WebsiteInfo};
use platform::objects::{Process, Progress, Timestamp as PlatformTimestamp, ToastManager, Window};
use util::error::Result;
use util::time::ToTicks;
use util::tracing::{ResultTraceExt, debug, info, trace};

use crate::desktop::{DesktopState, DimRequest, DimStatus, KillableProcessId};

/// Watcher to track [TriggeredAlert]s and [TriggeredReminder]s and take action on them.
pub struct Sentry {
    desktop_state: DesktopState,
    browser: ArcBrowser,
    mgr: AlertManager,
    website_actions: HashMap<String, WebsiteAction>,
}

#[derive(Clone, Debug)]
enum WebsiteAction {
    Dim(DimStatus),
    Kill,
}

impl WebsiteAction {
    fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (WebsiteAction::Dim(dim_status), WebsiteAction::Dim(other_dim_status)) => {
                WebsiteAction::Dim(dim_status.clone().into_merged(other_dim_status.clone()))
            }
            (WebsiteAction::Kill, _) => WebsiteAction::Kill,
            (_, WebsiteAction::Kill) => WebsiteAction::Kill,
        }
    }
}

impl Sentry {
    /// Create a new [Sentry] with the given [Cache] and [Database].
    pub fn new(desktop_state: DesktopState, browser: ArcBrowser, db: Database) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self {
            desktop_state,
            browser,
            mgr,
            website_actions: HashMap::new(),
        })
    }

    /// Run the [Sentry] to check for triggered alerts and reminders and take action on them.
    pub async fn run(&mut self, now: PlatformTimestamp) -> Result<()> {
        {
            let mut desktop_state = self.desktop_state.write().await;
            desktop_state.reset_dim_statuses();
            self.website_actions.clear();
            desktop_state.retain_cache().await?;
        }

        let alerts_hits = self.mgr.triggered_alerts(&now).await?;
        let mut dim_actions = HashMap::new();
        for triggered_alert in alerts_hits {
            self.handle_alert(&triggered_alert, &mut dim_actions, now)
                .await?;
        }

        {
            let mut desktop_state = self.desktop_state.write().await;

            // Merge dim actions from process/windows for alerts
            Self::merge_dim_actions(desktop_state.dim_statuses(), dim_actions);

            for (window, dim_status) in desktop_state.dim_statuses() {
                debug!(?dim_status, "alerts: dimming {window:?}");
                self.handle_dim_action(window, dim_status, now)?;
            }

            desktop_state
                .dim_statuses()
                .retain(|_, dim_status| !dim_status.is_empty());
        }

        // Push website actions to the browser backend for enforcement on tab changes.
        let browser_actions: HashMap<browser::BaseWebsiteUrl, BrowserAction> = self
            .website_actions
            .iter()
            .map(|(url_str, action)| {
                let base_url = WebsiteInfo::url_to_base_url(url_str);
                let browser_action = match action {
                    WebsiteAction::Kill => BrowserAction::CloseTab,
                    WebsiteAction::Dim(status) => BrowserAction::Dim {
                        opacity: status.opacity(now.to_ticks()),
                    },
                };
                (base_url, browser_action)
            })
            .collect();
        self.browser.set_actions(browser_actions)?;

        let reminder_hits = self.mgr.triggered_reminders(&now).await?;
        for triggered_reminder in reminder_hits {
            info!(?triggered_reminder, "send message");
            self.handle_reminder_message(
                &triggered_reminder.name,
                &triggered_reminder.reminder.message,
                triggered_reminder.reminder.threshold.into(),
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
        dim_actions: &mut HashMap<Window, DimStatus>,
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

                            let mut desktop_state = self.desktop_state.write().await;
                            desktop_state.remove_process(pid).await;
                        }
                        KillableProcessId::Aumid(aumid) => {
                            Process::kill_uwp(&aumid).await.warn();

                            let mut desktop_state = self.desktop_state.write().await;
                            desktop_state.remove_app(app).await;
                        }
                    }
                }
            }
            TriggerAction::Dim { duration } => {
                let (windows, websites) =
                    self.windows_and_websites_for_target(&alert.target).await?;

                let start = timestamp.unwrap_or(now.to_ticks());
                let dim_request = DimRequest {
                    by: alert.id.clone(),
                    start,
                    duration: *duration,
                };

                self.merge_website_actions(websites.into_iter().map(|url| {
                    (
                        url.to_string(),
                        WebsiteAction::Dim(DimStatus::with_one(dim_request.clone())),
                    )
                }));

                Self::merge_dim_actions(
                    dim_actions,
                    windows
                        .into_iter()
                        .map(|window| (window, DimStatus::with_one(dim_request.clone()))),
                );
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
    pub fn handle_dim_action(
        &self,
        window: &Window,
        dim_status: &DimStatus,
        now: PlatformTimestamp,
    ) -> Result<()> {
        let dim_level = dim_status.opacity(now.to_ticks());
        trace!(?dim_status, "dimming window {window:?} to {dim_level}");
        window.dim(dim_level).warn();
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

    async fn processes_and_websites_for_target(
        &mut self,
        target: &Target,
    ) -> Result<(
        Vec<(Ref<App>, KillableProcessId)>,
        Vec<browser::BaseWebsiteUrl>,
    )> {
        let target_apps = self.mgr.target_apps(target).await?;

        let desktop_state = self.desktop_state.read().await;
        let processes = target_apps
            .iter()
            .flat_map(move |app| {
                desktop_state
                    .platform_processes_for_app(app)
                    .into_iter()
                    .map(|pid| (app.clone(), pid))
            })
            .collect();

        let desktop_state = self.desktop_state.read().await;
        let websites = target_apps
            .iter()
            .flat_map(move |app| {
                desktop_state
                    .websites_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok((processes, websites))
    }

    async fn windows_and_websites_for_target(
        &mut self,
        target: &Target,
    ) -> Result<(Vec<Window>, Vec<browser::BaseWebsiteUrl>)> {
        let target_apps = self.mgr.target_apps(target).await?;

        let desktop_state = self.desktop_state.read().await;
        let windows = target_apps
            .iter()
            .flat_map(move |app| {
                desktop_state
                    .platform_windows_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();

        let desktop_state = self.desktop_state.read().await;
        let websites = target_apps
            .iter()
            .flat_map(move |app| {
                desktop_state
                    .websites_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok((windows, websites))
    }

    fn merge_dim_actions(
        dim_actions: &mut HashMap<Window, DimStatus>,
        new: impl IntoIterator<Item = (Window, DimStatus)>,
    ) {
        for (window, dim_status) in new {
            let existing = if let Some(existing) = dim_actions.get(&window) {
                existing.clone().into_merged(dim_status.clone())
            } else {
                dim_status.clone()
            };
            dim_actions.insert(window, existing);
        }
    }

    fn merge_website_actions(&mut self, new: impl IntoIterator<Item = (String, WebsiteAction)>) {
        for (url, action) in new {
            let action = if let Some(existing) = self.website_actions.get(&url) {
                action.merge(&existing.clone())
            } else {
                action
            };
            self.website_actions.insert(url, action);
        }
    }
}
