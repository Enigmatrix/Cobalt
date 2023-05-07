use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use common::errors::*;

use common::tracing::info;
use common::tracing::warn;
use data::db::Database;
use data::db::EntityInserter;
use data::db::FoundOrInserted;
use data::entities::App;
use data::entities::AppIdentity;
use data::entities::InteractionPeriod;
use data::entities::Session;
use data::entities::Usage;
use data::table::Ref;
use platform::objects::Process;
use platform::objects::ProcessId;
use platform::objects::{Timestamp, Window};
use platform::watchers::{InteractionStateChange, WindowSession};
use tokio::sync::mpsc::UnboundedSender;

use crate::app_info_resolver::*;

pub enum ProcessorEvent {
    WindowSession {
        at: Timestamp,
        change: WindowSession,
    },
    InteractionStateChange {
        change: InteractionStateChange,
    },
}

pub struct SessionDetails {
    session: Ref<Session>,
}

pub struct AppDetails {
    app: Ref<App>,
    cmd_line: Option<String>,
}

pub struct Processor<'a> {
    sessions: HashMap<WindowSession, SessionDetails>,
    apps: HashMap<ProcessId, AppDetails>,
    current_usage: Usage,
    inserter: EntityInserter<'a>,
    app_info_tx: UnboundedSender<AppInfoRequest>,
}

impl<'a> Processor<'a> {
    /// Create a new [Processor], which initializes it's first [Usage]
    pub fn new(
        foreground: Window,
        start: Timestamp,
        db: &'a mut Database,
        app_info_tx: UnboundedSender<AppInfoRequest>,
    ) -> Result<Self> {
        let mut sessions = HashMap::new();
        let mut apps = HashMap::new();
        let mut inserter = EntityInserter::from(db).context("create entity inserter")?;
        let title = foreground
            .title()
            .context("get foreground window title for processor")?;
        let session_details = Self::create_session_for_window(
            &mut apps,
            &mut inserter,
            &app_info_tx,
            &foreground,
            title.clone(),
        )
        .context("get session details for foreground window")?;
        let current_usage = Usage {
            id: Ref::default(),
            session: session_details.session.clone(),
            start: start.into(),
            end: start.into(),
        };
        sessions.insert(
            WindowSession {
                window: foreground,
                title,
            },
            session_details,
        );
        Ok(Self {
            sessions,
            apps,
            inserter,
            current_usage,
            app_info_tx,
        })
    }

    /// Handle a [ProcessorEvent]
    pub fn handle(&mut self, event: ProcessorEvent) -> Result<()> {
        match event {
            ProcessorEvent::WindowSession {
                at,
                change: window_session,
            } => {
                self.current_usage.end = at.into();
                self.inserter
                    .insert_usage(&self.current_usage)
                    .context("insert usage")?;

                info!(title = window_session.title, "switched window");

                let session_details = self
                    .sessions
                    .fallible_get_or_insert(window_session, |ws| {
                        Self::create_session_for_window(
                            &mut self.apps,
                            &mut self.inserter,
                            &self.app_info_tx,
                            &ws.window,
                            ws.title,
                        )
                        .context("create session details for window")
                    })
                    .context("get or create session details for window")?;
                self.current_usage = Usage {
                    id: Ref::default(),
                    session: session_details.session.clone(),
                    start: at.into(),
                    end: at.into(),
                };
            }
            ProcessorEvent::InteractionStateChange {
                change: InteractionStateChange::Active,
            } => {
                info!("active");
            }
            ProcessorEvent::InteractionStateChange {
                change:
                    InteractionStateChange::Idle {
                        mouseclicks,
                        keystrokes,
                        active_start,
                        idle_start,
                    },
            } => {
                info!("idle");
                self.inserter
                    .insert_interaction_period(&InteractionPeriod {
                        id: Ref::default(),
                        start: active_start.into(),
                        end: idle_start.into(),
                        mouseclicks,
                        keystrokes,
                    })
                    .context("insert interaction period")?;
            }
        }
        Ok(())
    }

    fn create_app_for_process(
        inserter: &mut EntityInserter<'a>,
        app_info_tx: &UnboundedSender<AppInfoRequest>,
        pid: ProcessId,
        window: &Window,
    ) -> Result<AppDetails> {
        let process = Process::new(pid).context("open process")?;

        let identity = if process.is_uwp(None).context("is process uwp")? {
            AppIdentity::Uwp {
                aumid: window.aumid().context("get window aumid")?,
            }
        } else {
            AppIdentity::Win32 {
                path: process.path().context("get process path")?,
            }
        };

        let cmd_line = match process.cmd_line() {
            Ok(v) => Some(v),
            Err(err) => {
                warn!(err = err.to_string(), identity = ?identity, "getting cmd_line failed");
                None
            }
        };

        let found_app = inserter
            .find_or_insert_app(&identity)
            .context("find or insert app")?;

        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                app_info_tx
                    .send(AppInfoRequest {
                        id: id.clone(),
                        app_identity: identity.clone(),
                    })
                    .context("send app info request")?;
                id
            }
        };
        Ok({
            AppDetails {
                app: app_id,
                cmd_line,
            }
        })
    }

    fn create_session_for_window(
        apps: &mut HashMap<ProcessId, AppDetails>,
        inserter: &mut EntityInserter<'a>,
        app_info_tx: &UnboundedSender<AppInfoRequest>,
        window: &Window,
        title: String,
    ) -> Result<SessionDetails> {
        let pid = window.pid().context("get window pid")?;
        let AppDetails { app, cmd_line } = apps
            .fallible_get_or_insert(pid, |pid| {
                Self::create_app_for_process(inserter, app_info_tx, pid, window)
                    .context("create app details for process")
            })
            .context("get or create app details for process")?;

        let mut session = Session {
            id: Ref::default(),
            app: app.clone(),
            title,
            cmd_line: cmd_line.clone(),
        };
        inserter
            .insert_session(&mut session)
            .context("insert session")?;

        Ok(SessionDetails {
            session: session.id,
        })
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
