use std::cell::RefCell;
use std::rc::Rc;

use data::db::{AlertManager, Database, TriggeredAlert};
use data::entities::{AlertEvent, ReminderEvent, Target, Timestamp, TriggerAction};
use platform::objects::{
    Process, ProcessId, Progress, Timestamp as PlatformTimestamp, ToastManager, Window,
};
use util::error::Result;
use util::tracing::ResultTraceExt;

use crate::cache::Cache;

// Tracks Alerts and Usages
pub struct Sentry<'a> {
    cache: Rc<RefCell<Cache>>,
    mgr: AlertManager<'a>,
}

// - move Engine's HashMap caches to a common component. Then Engine only does handle().
// - Sentry's main loop:
//     - get list of windows
//     - remove windows not in that list from the cache
//     - remove processes with no windows from the cache
//     - filter list of windows to those visible
//
//     - run sql query to get alerts & reminders
//     - for each alert:

impl<'a> Sentry<'a> {
    pub fn new(cache: Rc<RefCell<Cache>>, db: &'a mut Database) -> Result<Self> {
        let mgr = AlertManager::new(db)?;
        Ok(Self { cache, mgr })
    }

    pub fn run(&mut self, now: PlatformTimestamp) -> Result<()> {
        let alerts_hits = self.mgr.triggered_alerts(&now)?;
        for triggered_alert in alerts_hits {
            self.handle_alert(&triggered_alert, now)?;
        }

        let reminder_hits = self.mgr.triggered_reminders(&now)?;
        for triggered_reminder in reminder_hits {
            self.handle_reminder_message(
                &triggered_reminder.name,
                &triggered_reminder.reminder.message,
                triggered_reminder.reminder.threshold,
            )
            .warn();
            self.mgr.insert_reminder_event(&ReminderEvent {
                id: Default::default(),
                reminder: triggered_reminder.reminder.id,
                timestamp: now.into(),
            })?;
        }

        Ok(())
    }

    pub fn processes_for_target(&mut self, target: &Target) -> Result<Vec<ProcessId>> {
        let processes = self
            .mgr
            .target_apps(target)?
            .iter()
            .flat_map(move |app| {
                self.cache
                    .borrow()
                    .processes_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>() // ew
            })
            .collect();
        Ok(processes)
    }

    pub fn windows_for_target(&mut self, target: &Target) -> Result<Vec<Window>> {
        let window = self
            .mgr
            .target_apps(target)?
            .iter()
            .flat_map(move |app| {
                self.cache
                    .borrow()
                    .windows_for_app(app)
                    .cloned()
                    .collect::<Vec<_>>() // ew
            })
            .collect();
        Ok(window)
    }

    pub fn handle_alert(
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
                let processes = self.processes_for_target(&alert.target)?;
                for process in processes {
                    let process = Process::new_killable(process)?;
                    self.handle_kill_action(&process).warn();
                }
            }
            TriggerAction::Dim(dur) => {
                let windows = self.windows_for_target(&alert.target)?;
                let start = timestamp.unwrap_or(now.into());
                let end: Timestamp = now.into();
                let progress = (end - start) as f64 / (*dur as f64);
                for window in windows {
                    self.handle_dim_action(&window, progress.min(1.0f64)).warn();
                }
            }
            TriggerAction::Message(msg) => {
                if timestamp.is_none() {
                    self.handle_message_action(name, msg).warn();
                }
            }
        }
        if timestamp.is_none() {
            self.mgr.insert_alert_event(&AlertEvent {
                id: Default::default(),
                alert: alert.id.clone(),
                timestamp: now.into(),
            })?;
        }
        Ok(())
    }

    pub fn handle_kill_action(&self, process: &Process) -> Result<()> {
        process.kill(None)?;
        Ok(())
    }

    pub fn handle_message_action(&self, name: &str, msg: &str) -> Result<()> {
        ToastManager::show_basic(name, msg)?;
        Ok(())
    }

    pub fn handle_dim_action(&self, window: &Window, dim: f64) -> Result<()> {
        window.dim(1.0f64 - dim)?;
        Ok(())
    }

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
