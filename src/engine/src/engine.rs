use data::db::{DatabasePool, FoundOrInserted, UsageWriter};
use data::entities::{
    AppIdentity, InteractionPeriod, Ref, Session, SystemEvent as DataSystemEvent, Usage,
};
use platform::events::{
    ForegroundChangedEvent, ForegroundWindowSessionInfo, InteractionChangedEvent, SystemStateEvent,
};
use platform::objects::{Process, ProcessThreadId, Timestamp, Window};
use platform::web::{self, BaseWebsiteUrl, WebsiteInfo};
use scoped_futures::ScopedFutureExt;
use util::config::Config;
use util::error::{Context, Result};
use util::future::runtime::Handle;
use util::time::ToTicks;
use util::tracing::{ResultTraceExt, debug, info, trace};

use crate::desktop::{AppDetails, DesktopState, DesktopStateInner, SessionDetails};
use crate::foreground_window_session_async;
use crate::resolver::AppInfoResolver;

/// The main [Engine] that processes [Event]s and updates the [Database] with new [Usage]s, [Session]s and [App]s.
pub struct Engine {
    desktop_state: DesktopState,
    config: Config,
    web_state: web::State,
    current_usage: Usage,
    db_pool: DatabasePool,
    inserter: UsageWriter,
    spawner: Handle,
    active: bool,
}

/// Events that the [Engine] can handle.
pub enum Event {
    /// System Event
    System {
        /// System State Event
        event: SystemStateEvent,
        /// Last interaction before this system event - so that we can write/end the interaction period
        last_interaction: Option<InteractionChangedEvent>,
        /// Time of occurance
        now: Timestamp,
    },
    /// Foreground Changed Event
    ForegroundChanged(ForegroundChangedEvent),
    /// Interaction Changed Event
    InteractionChanged(InteractionChangedEvent),
    /// Tick Event (manual update)
    Tick(Timestamp),
}

/// Args for creating a new [Engine].
#[derive(Clone)]
pub struct EngineArgs {
    /// Desktop State
    pub desktop_state: DesktopState,
    /// Web State
    pub web_state: web::State,

    /// Config
    pub config: Config,

    /// Spawner for async tasks
    pub spawner: Handle,
    /// Database Pool
    pub db_pool: DatabasePool,

    /// Starting Session
    pub session: ForegroundWindowSessionInfo,
    /// Start time
    pub start: Timestamp,
}

impl Engine {
    /// Create a new [Engine], which initializes it's first [Usage]
    pub async fn new(options: EngineArgs) -> Result<Self> {
        let db = options.db_pool.get_db().await?;
        let inserter = UsageWriter::new(db)?;
        let mut ret = Self {
            desktop_state: options.desktop_state,
            config: options.config,
            web_state: options.web_state,
            db_pool: options.db_pool,
            inserter,
            // set a default value, then update it right after
            current_usage: Default::default(),
            active: true,
            spawner: options.spawner,
        };

        ret.current_usage = Usage {
            id: Default::default(),
            session_id: ret
                .get_session_details(options.session, options.start)
                .await?,
            start: options.start.to_ticks(),
            end: options.start.to_ticks(),
        };

        Ok(ret)
    }

    /// Handle an [Event]
    pub async fn handle(&mut self, event: Event) -> Result<()> {
        if let Event::System {
            event,
            now,
            last_interaction,
        } = &event
        {
            let prev = self.active;
            self.active = event.state.is_active();

            if prev && !self.active {
                // Stop usage watching, write last usage inside.
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)
                    .await?;
                // Save the interaction period if it exists.
                if let Some(interaction_period) = last_interaction {
                    self.inserter
                        .insert_interaction_period(&InteractionPeriod {
                            id: Default::default(),
                            start: interaction_period.start.to_ticks(),
                            end: interaction_period.end.to_ticks(),
                            mouse_clicks: interaction_period.mouse_clicks,
                            key_strokes: interaction_period.key_strokes,
                        })
                        .await?;
                }
            } else if !prev && self.active {
                let window_session =
                    foreground_window_session_async(&self.config, self.web_state.clone()).await?;
                self.current_usage = Usage {
                    id: Default::default(),
                    session_id: self.get_session_details(window_session, *now).await?,
                    start: now.to_ticks(),
                    end: now.to_ticks(),
                };
            }
            self.inserter
                .insert_system_event(&DataSystemEvent {
                    id: Default::default(),
                    timestamp: now.to_ticks(),
                    event: (&event.event).into(),
                })
                .await?;
            info!("system event processed: {:?}", event);
            return Ok(());
        }

        if !self.active {
            return Ok(());
        }

        match event {
            // handled above
            Event::System { .. } => unreachable!(),
            Event::ForegroundChanged(ForegroundChangedEvent { at, session }) => {
                debug!("fg switch: {:?}", session);

                self.current_usage.end = at.to_ticks();
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)
                    .await?;

                let session_result = self.get_session_details(session, at).await;

                // If we have an error getting the session, we don't change the current usage.
                // An alternative would be to insert some sort of 'invalid usage' marker.
                if let Ok(session) = &session_result {
                    self.current_usage = Usage {
                        id: Default::default(),
                        session_id: session.clone(),
                        start: at.to_ticks(),
                        end: at.to_ticks(),
                    };
                }

                session_result.warn();
            }
            Event::Tick(now) => {
                trace!("tick at {:?}", now);

                self.current_usage.end = now.to_ticks();
                self.inserter
                    .insert_or_update_usage(&mut self.current_usage)
                    .await?;
            }
            Event::InteractionChanged(InteractionChangedEvent {
                start,
                end,
                mouse_clicks,
                key_strokes,
            }) => {
                debug!("record interaction period {:?} - {:?}", start, end);

                self.inserter
                    .insert_interaction_period(&InteractionPeriod {
                        id: Default::default(),
                        start: start.to_ticks(),
                        end: end.to_ticks(),
                        mouse_clicks,
                        key_strokes,
                    })
                    .await?;
            }
        };
        Ok(())
    }

    /// Get the [Session] details for the given [WindowSession]
    async fn get_session_details(
        &mut self,
        session: ForegroundWindowSessionInfo,
        at: Timestamp,
    ) -> Result<Ref<Session>> {
        let mut desktop_state = self.desktop_state.write().await;

        let session_details = desktop_state
            .get_or_insert_session_for_window(session.clone(), |desktop_state| {
                async {
                    Self::create_session_for_window(
                        desktop_state,
                        self.db_pool.clone(),
                        &mut self.inserter,
                        &self.spawner,
                        session,
                        at,
                    )
                    .await
                }
                .scope_boxed()
            })
            .await?;
        // if !session_details.is_browser {
        //     ws.url = None;
        // }

        Ok(session_details.session.clone())
    }

    /// Create a [Session] for the given [Window]
    async fn create_session_for_window(
        desktop_state: &mut DesktopStateInner,
        db_pool: DatabasePool,
        inserter: &mut UsageWriter,
        spawner: &Handle,
        session: ForegroundWindowSessionInfo,
        at: Timestamp,
    ) -> Result<SessionDetails> {
        trace!(?session, "insert session");

        let ptid = session.window_session.window.ptid()?;
        let mut app = {
            let db_pool = db_pool.clone();
            let AppDetails { app, .. } = desktop_state
                .get_or_insert_app_for_ptid(ptid, |_| async {
                    Self::create_app_for_ptid(
                        inserter,
                        db_pool,
                        spawner,
                        ptid,
                        &session.window_session.window,
                        session.fetched_path,
                        at,
                    )
                    .await
                })
                .await?;
            app.clone()
        };

        if let Some(url) = &session.window_session.url {
            let base_url = WebsiteInfo::url_to_base_url(url);
            let AppDetails { app: web_app, .. } = desktop_state
                .get_or_insert_website_for_base_url(base_url.clone(), |_| async {
                    Self::create_app_for_base_url(inserter, db_pool, spawner, base_url, at).await
                })
                .await?;
            app = web_app.clone();
        }

        let mut session = Session {
            id: Default::default(),
            app_id: app,
            title: session.window_session.title,
            url: session.window_session.url,
        };
        inserter.insert_session(&mut session).await?;

        Ok(SessionDetails {
            session: session.id,
            ptid,
        })
    }

    /// Create an [App] for the given [ProcessThreadId] and [Window]
    async fn create_app_for_ptid(
        inserter: &mut UsageWriter,
        db_pool: DatabasePool,
        spawner: &Handle,
        ptid: ProcessThreadId,
        window: &Window,
        fetched_path: Option<String>,
        at: Timestamp,
    ) -> Result<AppDetails> {
        trace!(?window, ?ptid, "create/find app for process");

        let process = Process::new(ptid.pid)?;
        let path = if let Some(fetched_path) = fetched_path {
            fetched_path
        } else {
            process.path()?
        };

        let identity = if let Some(aumid) = process.aumid()? {
            debug!("desktop aumid: {aumid}");
            AppIdentity::Uwp { aumid }
        } else if process.is_uwp(Some(&path))? {
            let aumid = window.aumid()?;
            debug!("store aumid: {aumid}");
            AppIdentity::Uwp { aumid }
        } else {
            AppIdentity::Win32 { path }
        };

        let app = AppInfoResolver::default_from(&identity, at);
        let found_app = inserter.find_or_insert_app(&app).await?;
        let _identity = identity.clone();
        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                {
                    info!(?window, ?ptid, "inserted app");

                    let id = id.clone();

                    spawner.spawn(async move {
                        AppInfoResolver::update_app(db_pool, id.clone(), _identity.clone())
                            .await
                            .with_context(|| format!("update app({id:?}, {_identity:?}) with info"))
                            .error();
                    });
                }

                id
            }
        };
        Ok(AppDetails {
            app: app_id,
            identity,
        })
    }

    async fn create_app_for_base_url(
        inserter: &mut UsageWriter,
        db_pool: DatabasePool,
        spawner: &Handle,
        base_url: BaseWebsiteUrl,
        at: Timestamp,
    ) -> std::result::Result<AppDetails, util::error::Error> {
        trace!(?base_url, "create/find app for base url");

        let identity = AppIdentity::Website {
            base_url: base_url.to_string(),
        };
        let app = AppInfoResolver::default_from(&identity, at);
        let found_app = inserter.find_or_insert_app(&app).await?;

        let _identity = identity.clone();
        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                {
                    info!(?base_url, "inserted app for website");

                    let id = id.clone();

                    spawner.spawn(async move {
                        AppInfoResolver::update_app(db_pool, id.clone(), _identity.clone())
                            .await
                            .with_context(|| {
                                format!("update app({id:?}, {_identity:?}) (website) with info")
                            })
                            .error();
                    });
                }

                id
            }
        };
        // this is a website, not a browser
        Ok(AppDetails {
            app: app_id,
            identity,
        })
    }
}
