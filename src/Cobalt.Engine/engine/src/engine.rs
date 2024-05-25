use std::cell::RefCell;
use std::rc::Rc;

use data::db::{Database, FoundOrInserted, UsageWriter};
use data::entities::{AppIdentity, InteractionPeriod, Ref, Session, Usage};
use platform::events::{ForegroundChangedEvent, InteractionChangedEvent, WindowSession};
use platform::objects::{Process, ProcessId, Timestamp, Window};
use util::config::Config;
use util::error::Result;
use util::future::task::LocalSpawnExt;
use util::tracing::info;

use crate::cache::{AppDetails, Cache, SessionDetails};
use crate::resolver::AppInfoResolver;

pub struct Engine<'a, S: LocalSpawnExt> {
    cache: Rc<RefCell<Cache>>,
    current_usage: Usage,
    active_period_start: Timestamp,
    config: Config,
    inserter: UsageWriter<'a>,
    spawner: S,
}

pub enum Event {
    ForegroundChanged(ForegroundChangedEvent),
    InteractionChanged(InteractionChangedEvent),
    Tick(Timestamp),
}

impl<'a, S: LocalSpawnExt> Engine<'a, S> {
    /// Create a new [Engine], which initializes it's first [Usage]
    pub async fn new(
        cache: Rc<RefCell<Cache>>,
        foreground: Window,
        start: Timestamp,
        config: Config,
        db: &'a mut Database,
        spawner: S,
    ) -> Result<Self> {
        let inserter = UsageWriter::new(db)?;
        let title = foreground.title()?;
        let ws = WindowSession {
            window: foreground.clone(),
            title: title.clone(),
        };

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
            session: ret.get_session_details(ws)?,
            start: start.into(),
            end: start.into(),
        };

        Ok(ret)
    }

    pub async fn handle(&mut self, event: Event) -> Result<()> {
        match event {
            Event::ForegroundChanged(ForegroundChangedEvent { at, session }) => {
                self.current_usage.end = at.into();
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)?;

                info!("Foreground changed to {:?}", session);

                let session = self.get_session_details(session)?;

                self.current_usage = Usage {
                    id: Default::default(),
                    session,
                    start: at.into(),
                    end: at.into(),
                };
            }
            Event::Tick(now) => {
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
                        mouseclicks: recorded_mouse_clicks,
                        keystrokes: recorded_key_presses,
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

    fn get_session_details(&mut self, ws: WindowSession) -> Result<Ref<Session>> {
        let mut borrow = self.cache.borrow_mut();
        let session_details = borrow.get_or_insert_session_for_window(ws.clone(), |cache| {
            Self::create_session_for_window(
                cache,
                &self.config,
                &mut self.inserter,
                &self.spawner,
                &ws.window,
                ws.title,
            )
        })?;

        Ok(session_details.session.clone())
    }

    fn create_app_for_process(
        inserter: &mut UsageWriter<'a>,
        config: &Config,
        spawner: &S,
        pid: ProcessId,
        window: &Window,
    ) -> Result<AppDetails> {
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
                    let config = config.clone();
                    let id = id.clone();

                    spawner.spawn_local(async move {
                        AppInfoResolver::update_app(&config, id, identity)
                            .await
                            .expect("update app with info")
                    })?;
                }

                id
            }
        };
        Ok(AppDetails { app: app_id })
    }

    fn create_session_for_window(
        cache: &mut Cache,
        config: &Config,
        inserter: &mut UsageWriter<'a>,
        spawner: &S,
        window: &Window,
        title: String,
    ) -> Result<SessionDetails> {
        let pid = window.pid()?;
        let AppDetails { app } = cache.get_or_insert_app_for_process(pid, |_| {
            Self::create_app_for_process(inserter, config, spawner, pid, window)
        })?;

        let mut session = Session {
            id: Default::default(),
            app: app.clone(),
            title,
        };
        inserter.insert_session(&mut session)?;

        Ok(SessionDetails {
            session: session.id,
            pid,
        })
    }
}
