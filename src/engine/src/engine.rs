use std::sync::Arc;

use data::db::{Database, DatabasePool, FoundOrInserted, UsageWriter};
use data::entities::{
    AppIdentity, InteractionPeriod, Ref, Session, SystemEvent as DataSystemEvent, Usage,
};
use platform::events::{
    ForegroundChangedEvent, InteractionChangedEvent, SystemStateEvent, WindowSession,
};
use platform::objects::{
    BaseWebsiteUrl, BrowserDetector, Process, ProcessThreadId, Timestamp, WebsiteInfo, Window,
};
use scoped_futures::ScopedFutureExt;
use util::config;
use util::error::{Context, Result};
use util::future::runtime::Handle;
use util::future::sync::Mutex;
use util::time::ToTicks;
use util::tracing::{debug, info, trace, ResultTraceExt};

use crate::cache::{AppDetails, Cache, SessionDetails};
use crate::foreground_window_session;
use crate::resolver::AppInfoResolver;

/// The main [Engine] that processes [Event]s and updates the [Database] with new [Usage]s, [Session]s and [App]s.
pub struct Engine {
    cache: Arc<Mutex<Cache>>,
    current_usage: Usage,
    db_pool: DatabasePool,
    inserter: UsageWriter,
    spawner: Handle,
    active: bool,
}

/// Events that the [Engine] can handle.
pub enum Event {
    System {
        event: SystemStateEvent,
        last_interaction: Option<InteractionChangedEvent>,
        now: Timestamp,
    },
    ForegroundChanged(ForegroundChangedEvent),
    InteractionChanged(InteractionChangedEvent),
    Tick(Timestamp),
}

impl Engine {
    /// Create a new [Engine], which initializes it's first [Usage]
    pub async fn new(
        cache: Arc<Mutex<Cache>>,
        db_pool: DatabasePool,
        foreground: WindowSession,
        start: Timestamp,
        db: Database,
        spawner: Handle,
    ) -> Result<Self> {
        let inserter = UsageWriter::new(db)?;
        let mut ret = Self {
            cache,
            db_pool,
            inserter,
            // set a default value, then update it right after
            current_usage: Default::default(),
            active: true,
            spawner,
        };

        ret.current_usage = Usage {
            id: Default::default(),
            session_id: ret.get_session_details(foreground, start).await?,
            start: start.to_ticks(),
            end: start.to_ticks(),
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
                // Restart usage watching.
                let config = config::get_config()?;
                let foreground = foreground_window_session(&config)?;
                self.current_usage = Usage {
                    id: Default::default(),
                    session_id: self.get_session_details(foreground, *now).await?,
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
        mut ws: WindowSession,
        at: Timestamp,
    ) -> Result<Ref<Session>> {
        let mut cache = self.cache.lock().await;

        // If window is browser (assume true if unknown), then we need the url.
        // Else set the url to None.
        if !cache.is_browser(&ws.window).unwrap_or(true) {
            ws.url = None;
        }

        let session_details = cache
            .get_or_insert_session_for_window(ws.clone(), |cache| {
                async {
                    Self::create_session_for_window(
                        cache,
                        self.db_pool.clone(),
                        &mut self.inserter,
                        &self.spawner,
                        ws,
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
        cache: &mut Cache,
        db_pool: DatabasePool,
        inserter: &mut UsageWriter,
        spawner: &Handle,
        mut ws: WindowSession,
        at: Timestamp,
    ) -> Result<SessionDetails> {
        trace!(?ws, "insert session");

        let ptid = ws.window.ptid()?;
        let (mut app, is_browser) = {
            let db_pool = db_pool.clone();
            let AppDetails { app, is_browser } = cache
                .get_or_insert_app_for_ptid(ptid, |_| async {
                    Self::create_app_for_ptid(inserter, db_pool, spawner, ptid, &ws.window, at)
                        .await
                })
                .await?;
            (app.clone(), *is_browser)
        };

        if !is_browser {
            ws.url = None;
        }

        if let Some(url) = &ws.url {
            let base_url = WebsiteInfo::url_to_base_url(url).context("url to base url")?;
            let AppDetails { app: web_app, .. } = cache
                .get_or_insert_website_for_base_url(base_url.clone(), |_| async {
                    Self::create_app_for_base_url(inserter, db_pool, spawner, base_url, at).await
                })
                .await?;
            app = web_app.clone();
        }

        let mut session = Session {
            id: Default::default(),
            app_id: app,
            title: ws.title,
            url: ws.url,
        };
        inserter.insert_session(&mut session).await?;

        Ok(SessionDetails {
            session: session.id,
            ptid,
            is_browser,
        })
    }

    /// Create an [App] for the given [ProcessThreadId] and [Window]
    async fn create_app_for_ptid(
        inserter: &mut UsageWriter,
        db_pool: DatabasePool,
        spawner: &Handle,
        ptid: ProcessThreadId,
        window: &Window,
        at: Timestamp,
    ) -> Result<AppDetails> {
        trace!(?window, ?ptid, "create/find app for process");

        let process = Process::new(ptid.pid)?;
        let path = process.path()?;
        let is_browser = BrowserDetector::is_browser(&path);

        let identity = if process.is_uwp(Some(&path))? {
            AppIdentity::Uwp {
                aumid: window.aumid()?,
            }
        } else {
            AppIdentity::Win32 { path }
        };

        let found_app = inserter.find_or_insert_app(&identity, at).await?;
        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                {
                    info!(?window, ?ptid, "inserted app");

                    let id = id.clone();

                    spawner.spawn(async move {
                        AppInfoResolver::update_app(db_pool, id.clone(), identity.clone())
                            .await
                            .with_context(|| {
                                format!("update app({:?}, {:?}) with info", id, identity)
                            })
                            .error();
                    });
                }

                id
            }
        };
        Ok(AppDetails {
            app: app_id,
            is_browser,
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
        let found_app = inserter.find_or_insert_app(&identity, at).await?;

        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                {
                    info!(?base_url, "inserted app for website");

                    let id = id.clone();

                    spawner.spawn(async move {
                        AppInfoResolver::update_app(db_pool, id.clone(), identity.clone())
                            .await
                            .with_context(|| {
                                format!("update app({:?}, {:?}) (website) with info", id, identity)
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
            is_browser: false,
        })
    }
}
