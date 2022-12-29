use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use data::db::*;
use data::models;
use platform::events::{Event as PlatformEvent, InteractionStateChange};
use platform::objects::{PidTid, Process, ProcessId, Timestamp, Window};
use utils::channels::Sender;
use utils::errors::*;

pub struct ProcessDetails {
    pub app: models::Ref<models::App>,
    pub process: Process,
}

pub struct SessionDetails {
    pub session: models::Ref<models::Session>,
    pub window: Window,
}

pub struct Processor<'a> {
    processes: HashMap<ProcessId, ProcessDetails>,
    windows: HashMap<Window, SessionDetails>,

    db: Database<'a>,
    app_info_tx: Sender<models::Ref<models::App>>,

    current_usage: Option<models::Usage>,
    interaction_start: Timestamp,
}

#[derive(Debug)]
pub enum Event {
    Platform(PlatformEvent),
    AppInfoUpdate(models::App),
}

impl<'a> Processor<'a> {
    pub fn new(
        db: Database<'a>,
        app_info_tx: Sender<models::Ref<models::App>>,
        now: Timestamp,
    ) -> Self {
        Self {
            processes: HashMap::new(),
            windows: HashMap::new(),

            db,
            app_info_tx,

            current_usage: None,
            interaction_start: now,
        }
    }

    pub fn create_process_details(
        pid: ProcessId,
        window: Window,
        db: &mut Database<'_>,
        app_info_tx: &Sender<models::Ref<models::App>>,
    ) -> Result<ProcessDetails> {
        let process = Process::new(pid).with_context(|| format!("create process for pid={pid}"))?;
        let path = process.path().context("get process path")?;
        let app_identity = if process
            .is_uwp(Some(&path))
            .context("check if process is uwp")?
        {
            models::AppIdentity::UWP {
                aumid: window.aumid().context("get aumid for window")?,
            }
        } else {
            models::AppIdentity::Win32 { path }
        };

        let app = match db
            .find_or_insert_empty_app(&app_identity)
            .with_context(|| format!("find or insert empty app for {:?}", &app_identity))?
        {
            FoundOrInserted::Found(existing_app) => existing_app.id,
            FoundOrInserted::Inserted(new_app) => {
                app_info_tx
                    .send(new_app.clone())
                    .context("send app info request msg")?;
                new_app
            }
        };

        Ok(ProcessDetails { app, process })
    }

    pub fn create_session_details(
        window: Window,
        processes: &mut HashMap<ProcessId, ProcessDetails>,
        db: &mut Database<'_>,
        app_info_tx: &Sender<models::Ref<models::App>>,
    ) -> Result<SessionDetails> {
        let title = window.title().context("get window title")?; // TODO we are getting window title too many times ... maybe make it part of platform::Event?
        let PidTid { pid, .. } = window.pid_tid().context("get window pid")?;

        let ProcessDetails { app, process } = processes
            .fallible_get_or_insert(pid, |pid| {
                Self::create_process_details(pid, window.clone(), db, app_info_tx)
            })
            .context("get or insert process details")?;

        let mut session = models::Session {
            id: models::Ref::default(),
            app: app.clone(),
            title,
            cmd_line: Some(process.cmd_line().context("get command line")?), //TODO fallible!
        };
        db.insert_session(&mut session).context("insert session")?;
        let session = session.id;
        Ok(SessionDetails { session, window })
    }

    pub fn process(&mut self, ev: Event) -> Result<()> {
        match ev {
            Event::Platform(ev) => match ev {
                PlatformEvent::ForegroundSwitch { at, window } => {
                    // TODO remove extaneous processes and windows

                    if let Some(current_usage) = &mut self.current_usage {
                        current_usage.end = at.into();
                        self.db
                            .insert_usage(current_usage)
                            .context("insert usage")?;
                    }

                    let SessionDetails { session, .. } = self
                        .windows
                        .fallible_get_or_insert(window, |window| {
                            Self::create_session_details(
                                window,
                                &mut self.processes,
                                &mut self.db,
                                &self.app_info_tx,
                            )
                        })
                        .context("get or insert session details")?;

                    self.current_usage = Some(models::Usage {
                        id: models::Ref::default(),
                        session: session.clone(),
                        start: at.into(),
                        end: at.into(),
                    });
                }
                PlatformEvent::InteractionStateChange { at, change } => match change {
                    InteractionStateChange::Active => {
                        self.interaction_start = at;
                    }
                    InteractionStateChange::Idle {
                        mouseclicks,
                        keystrokes,
                    } => {
                        let mut interaction_period = models::InteractionPeriod {
                            id: models::Ref::default(),
                            start: self.interaction_start.into(),
                            end: at.into(),
                            mouseclicks: mouseclicks as u64,
                            keystrokes: keystrokes as u64,
                        };
                        self.db
                            .insert_interaction_period(&mut interaction_period)
                            .context("insert interaction period")?;
                    }
                },
            },
            Event::AppInfoUpdate(app) => {
                // idk actually
            }
        }
        Ok(())
    }
}

trait HashMapExt<K, V> {
    fn fallible_get_or_insert<F: FnOnce(K) -> Result<V>>(
        &mut self,
        key: K,
        create: F,
    ) -> Result<&mut V>;
}

impl<K: Eq + Hash + Clone, V> HashMapExt<K, V> for HashMap<K, V> {
    fn fallible_get_or_insert<F: FnOnce(K) -> Result<V>>(
        &mut self,
        key: K,
        create: F,
    ) -> Result<&mut V> {
        match self.entry(key.clone()) {
            Entry::Occupied(occ) => Ok(occ.into_mut()),
            Entry::Vacant(vac) => Ok(vac.insert(create(key).context("creating new entry")?)),
        }
    }
}
