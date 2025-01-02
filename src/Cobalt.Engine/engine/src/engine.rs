use std::cell::RefCell;
use std::rc::Rc;

use data::db::{Database, FoundOrInserted, UsageWriter};
use data::entities::{AppIdentity, InteractionPeriod, Ref, Session, Usage};
use platform::events::{ForegroundChangedEvent, InteractionChangedEvent, WindowSession};
use platform::objects::{Process, ProcessId, Timestamp, Window};
use util::config::Config;
use util::error::{Context, Result};
use util::future::task::LocalSpawnExt;
use util::tracing::{info, trace, ResultTraceExt};

use crate::cache::{AppDetails, Cache, SessionDetails};
use crate::resolver::AppInfoResolver;

/// The main [Engine] that processes [Event]s and updates the [Database] with new [Usage]s, [Session]s and [App]s.
pub struct Engine<'a, S: LocalSpawnExt> {
    cache: Rc<RefCell<Cache>>,
    current_usage: Usage,
    active_period_start: Timestamp,
    config: Config,
    inserter: UsageWriter<'a>,
    spawner: S,
}

/// Events that the [Engine] can handle.
pub enum Event {
    ForegroundChanged(ForegroundChangedEvent),
    InteractionChanged(InteractionChangedEvent),
    Tick(Timestamp),
}

impl<'a, S: LocalSpawnExt> Engine<'a, S> {
    /// Create a new [Engine], which initializes it's first [Usage]
    pub async fn new(
        cache: Rc<RefCell<Cache>>,
        foreground: WindowSession,
        start: Timestamp,
        config: Config,
        db: &'a mut Database,
        spawner: S,
    ) -> Result<Self> {
        let inserter = UsageWriter::new(db)?;
        let mut ret = Self {
            cache,
            config,
            inserter,
            active_period_start: start,
            // set a default value, then update it right after
            current_usage: Default::default(),
            spawner,
        };

        ret.current_usage = Usage {
            id: Default::default(),
            session_id: ret.get_session_details(foreground)?,
            start: start.into(),
            end: start.into(),
        };

        Ok(ret)
    }

    /// Handle an [Event]
    pub async fn handle(&mut self, event: Event) -> Result<()> {
        match event {
            Event::ForegroundChanged(ForegroundChangedEvent { at, session }) => {
                info!("fg switch: {:?}", session);

                self.current_usage.end = at.into();
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)?;

                let session_result = self.get_session_details(session);

                // If we have an error getting the session, we don't change the current usage.
                // An alternative would be to insert some sort of 'invalid usage' marker.
                if let Ok(session) = &session_result {
                    self.current_usage = Usage {
                        id: Default::default(),
                        session_id: session.clone(),
                        start: at.into(),
                        end: at.into(),
                    };
                }

                session_result.warn();
            }
            Event::Tick(now) => {
                trace!("tick at {:?}", now);

                self.current_usage.end = now.into();
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)?;
            }
            Event::InteractionChanged(InteractionChangedEvent::BecameIdle {
                at,
                recorded_mouse_clicks,
                recorded_key_presses,
            }) => {
                info!("became idle at {:?}", at);

                self.inserter
                    .insert_interaction_period(&InteractionPeriod {
                        id: Default::default(),
                        start: self.active_period_start.into(),
                        end: at.into(),
                        mouse_clicks: recorded_mouse_clicks,
                        key_strokes: recorded_key_presses,
                    })?;
                // don't need to update active_period_start, as it will be updated when we become active again
            }
            Event::InteractionChanged(InteractionChangedEvent::BecameActive { at }) => {
                info!("became active at {:?}", at);

                self.active_period_start = at;
            }
        };
        Ok(())
    }

    /// Get the [Session] details for the given [WindowSession]
    fn get_session_details(&mut self, ws: WindowSession) -> Result<Ref<Session>> {
        let mut borrow = self.cache.borrow_mut();
        let session_details = borrow.get_or_insert_session_for_window(ws.clone(), |cache| {
            Self::create_session_for_window(
                cache,
                &self.config,
                &mut self.inserter,
                &self.spawner,
                ws,
            )
        })?;

        Ok(session_details.session.clone())
    }

    /// Create a [Session] for the given [Window]
    fn create_session_for_window(
        cache: &mut Cache,
        config: &Config,
        inserter: &mut UsageWriter<'a>,
        spawner: &S,
        ws: WindowSession,
    ) -> Result<SessionDetails> {
        info!(?ws, "insert session");

        let pid = ws.window.pid()?;
        let AppDetails { app } = cache.get_or_insert_app_for_process(pid, |_| {
            Self::create_app_for_process(inserter, config, spawner, pid, &ws.window)
        })?;

        let mut session = Session {
            id: Default::default(),
            app_id: app.clone(),
            title: ws.title,
        };
        inserter.insert_session(&mut session)?;

        Ok(SessionDetails {
            session: session.id,
            pid,
        })
    }

    /// Create an [App] for the given [ProcessId] and [Window]
    fn create_app_for_process(
        inserter: &mut UsageWriter<'a>,
        config: &Config,
        spawner: &S,
        pid: ProcessId,
        window: &Window,
    ) -> Result<AppDetails> {
        trace!(?window, ?pid, "create/find app for process");

        let process = Process::new(pid)?;

        let path = process.path()?;
        let identity = if process.is_uwp(Some(&path))? {
            AppIdentity::Uwp {
                aumid: window.aumid()?,
            }
        } else {
            AppIdentity::Win32 { path }
        };

        let found_app = inserter.find_or_insert_app(&identity)?;
        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                {
                    info!(?window, ?pid, "inserted app");

                    let config = config.clone();
                    let id = id.clone();

                    spawner.spawn_local(async move {
                        AppInfoResolver::update_app(&config, id, identity)
                            .await
                            .context("update app with info")
                            .error();
                    })?;
                }

                id
            }
        };
        Ok(AppDetails { app: app_id })
    }
}
