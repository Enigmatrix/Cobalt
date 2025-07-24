use std::collections::HashMap;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{
    AlertEvent, App, Duration, Reason, Ref, ReminderEvent, Target, TriggerAction,
};
use platform::objects::{Process, Progress, Timestamp as PlatformTimestamp, ToastManager, Window};
use platform::web;
use util::config::Config;
use util::error::Result;
use util::time::ToTicks;
use util::tracing::{ResultTraceExt, debug, info};

use crate::desktop::{DesktopState, DimRequest, DimStatus, KillableProcessId};

/// Watcher to track [TriggeredAlert]s and [TriggeredReminder]s and take action on them.
pub struct Sentry {
    config: Config,
    desktop_state: DesktopState,
    web_state: web::State,
    mgr: AlertManager,
    // Invariant: the website_actions map is *wholly* updated. That is, run() executes
    // and updates this after clearing it with *all* alerts' websites. This is easily
    // maintained since the two places its used - run() and handle_web_change() are both &mut self.
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
    pub fn new(
        config: Config,
        desktop_state: DesktopState,
        web_state: web::State,
        db: Database,
    ) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self {
            config,
            desktop_state,
            web_state,
            mgr,
            website_actions: HashMap::new(),
        })
    }

    /// Run Alert Actions for the browser window matching the given [web::Changed].
    pub async fn handle_web_change(&mut self, web_change: web::Changed) -> Result<()> {
        let now = PlatformTimestamp::now();
        let detect = web::Detect::new()?;
        let web::Changed {
            window,
            new_url,
            prev_url,
            is_incognito,
        } = web_change;

        // skip if window is dead or minimized
        if window.ptid().is_err() || window.is_minimized() {
            return Ok(());
        }

        // skip if incognito and not tracking incognito
        if is_incognito && !self.config.track_incognito() {
            return Ok(());
        }

        let base_url = web::WebsiteInfo::url_to_base_url(&new_url)?.to_string();
        let prev_base_url = web::WebsiteInfo::url_to_base_url(&prev_url)?.to_string();

        if let Some(action) = self.website_actions.get(&base_url) {
            match action {
                WebsiteAction::Dim(dim_status) => {
                    let mut desktop_state = self.desktop_state.write().await;
                    let current_dim_status = desktop_state.dim_status_mut(&window);

                    // remove previous dim requests for the previous url if there are any
                    if let Some(WebsiteAction::Dim(prev_website_dim_status)) =
                        self.website_actions.get(&prev_base_url)
                    {
                        current_dim_status.subtract(prev_website_dim_status);
                    }
                    // add new dim requests for the new url
                    current_dim_status.merge(dim_status);

                    debug!(?current_dim_status, "tab switch: dimming {window:?}",);

                    let dim_level = current_dim_status.opacity(now.to_ticks());
                    self.handle_dim_action(&window, dim_level).warn();
                }
                WebsiteAction::Kill => {
                    let element = {
                        let web_state = self.web_state.read().await;

                        let Some(state) = web_state.get_browser_window(&window)? else {
                            return Ok(());
                        };
                        state.extracted_elements.window_element.resolve()?
                    };
                    detect.close_current_tab(&element).warn();
                }
            }
        } else {
            // if we took a dim action for the previous url, we need to remove it
            if let Some(WebsiteAction::Dim(dim_status)) = self.website_actions.get(&prev_base_url) {
                let mut desktop_state = self.desktop_state.write().await;
                let current_dim_status = desktop_state.dim_status_mut(&window);
                current_dim_status.subtract(dim_status);

                debug!(
                    ?current_dim_status,
                    "tab switch: removing dim status for {window:?}"
                );
                let dim_level = current_dim_status.opacity(now.to_ticks());
                self.handle_dim_action(&window, dim_level).warn();
            }
        }

        Ok(())
    }

    /// Run the [Sentry] to check for triggered alerts and reminders and take action on them.
    pub async fn run(&mut self, now: PlatformTimestamp) -> Result<()> {
        // Retain alive entries in desktop_state before we start processing
        // - avoids dimming dead windows / killing dead processes
        {
            let mut desktop_state = self.desktop_state.write().await;

            // reset dim statuses - we will repopulate them with the alerts
            desktop_state.reset_dim_statuses();
            self.website_actions.clear();

            desktop_state.retain_cache().await?;
        }

        let alerts_hits = self.mgr.triggered_alerts(&now).await?;
        // Dim actions are stored here so we can dim windows later - they do not care about website yet!
        let mut dim_actions = HashMap::new();
        for triggered_alert in alerts_hits {
            self.handle_alert(&triggered_alert, &mut dim_actions, now)
                .await?;
        }

        {
            let mut desktop_state = self.desktop_state.write().await;
            let web_state = self.web_state.read().await;

            // Merge dim actions from process/windows for alerts
            Self::merge_dim_actions(desktop_state.dim_statuses(), dim_actions);
            // Merge dim actions from browser windows' last urls
            Self::merge_dim_actions(
                desktop_state.dim_statuses(),
                web_state
                    .browser_windows
                    .iter()
                    .flat_map(|(window, state)| state.iter().map(move |state| (window, state)))
                    .flat_map(|(window, state)| {
                        // skip if incognito and not tracking incognito
                        if !self.config.track_incognito() && state.is_incognito {
                            return None;
                        }

                        let base_url = web::WebsiteInfo::url_to_base_url(&state.last_url)
                            .map(Some)
                            .warn()?;
                        let Some(WebsiteAction::Dim(dim_status)) =
                            self.website_actions.get(&base_url.to_string())
                        else {
                            return None;
                        };
                        Some((window.clone(), dim_status.clone()))
                    }),
            );

            for (window, dim_status) in desktop_state.dim_statuses() {
                debug!(?dim_status, "alerts: dimming {window:?}");

                let dim_level = dim_status.opacity(now.to_ticks());
                self.handle_dim_action(window, dim_level)?;
            }

            // remove empty dim statuses
            desktop_state
                .dim_statuses()
                .retain(|_, dim_status| !dim_status.is_empty());
        }

        // TODO: logging move to handle_dim_action
        //
        // for window in windows {
        //     if dim_level == 1.0f64 {
        //         info!(?alert, "start dimming window {:?}", window);
        //     } else if dim_level == MIN_DIM_LEVEL && progress <= 1.01f64 {
        //         // allow for floating point imprecision. check if we reach ~100% progress
        //         info!(?alert, "max dim window reached for {:?}", window);
        //     } else {
        //         debug!(?alert, "dimming window {:?} to {}", window, dim_level);
        //     }
        //     self.handle_dim_action(&window, dim_level).warn();
        // }

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

                // set websites to kill. kill takes precedence
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

                // set websites to dim. kill takes precedence
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
    pub fn handle_dim_action(&self, window: &Window, dim: f64) -> Result<()> {
        window.dim(dim).warn();
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
    ) -> Result<(Vec<(Ref<App>, KillableProcessId)>, Vec<web::BaseWebsiteUrl>)> {
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

        // yes, we lock twice here ...
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

    /// Get the [Window]s and websites for the given [Target],
    async fn windows_and_websites_for_target(
        &mut self,
        target: &Target,
    ) -> Result<(Vec<Window>, Vec<web::BaseWebsiteUrl>)> {
        let target_apps = self.mgr.target_apps(target).await?;

        let desktop_state = self.desktop_state.read().await;
        let windows = target_apps
            .iter()
            .flat_map(move |app| {
                desktop_state
                    .platform_windows_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>() // ew
            })
            .collect();

        // yes, we lock twice here ...
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
