use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::Future;

use data::db::{Database, FoundOrInserted, UsageWriter};
use data::entities::{App, AppIdentity, Ref, Session, Usage};
use platform::events::{ForegroundChangedEvent, InteractionChangedEvent};
use platform::objects::{Process, ProcessId, Timestamp, Window};
use std::hash::Hash;
use util::channels::Sender;
use util::error::Result;

use util::tracing::info;

use crate::resolver::AppInfoResolverRequest;

pub struct Engine<'a> {
    // TODO this might be a bad idea, the HWND might be reused by Windows,
    // so another window could be running with the same HWND after the first one closed...
    sessions: HashMap<WindowSession, SessionDetails>,
    // TODO this might be a bad idea, the ProcessId might be reused by Windows,
    // so another app could be running with the same pid after the first one closed...
    apps: HashMap<ProcessId, AppDetails>,
    current_usage: Usage,
    inserter: UsageWriter<'a>,
    resolver: Sender<AppInfoResolverRequest>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    window: Window,
    title: String,
}

pub struct SessionDetails {
    session: Ref<Session>,
}

pub struct AppDetails {
    app: Ref<App>,
}

pub enum Event {
    ForegroundChanged(ForegroundChangedEvent),
    InteractionChanged(InteractionChangedEvent),
}

impl<'a> Engine<'a> {
    /// Create a new [Engine], which initializes it's first [Usage]
    pub async fn new(
        foreground: Window,
        start: Timestamp,
        db: &'a mut Database,
        resolver: Sender<AppInfoResolverRequest>,
    ) -> Result<Self> {
        let mut sessions = HashMap::new();
        let mut apps = HashMap::new();
        let mut inserter = UsageWriter::new(db)?;
        let title = foreground.title()?;
        let session_details = Self::create_session_for_window(
            &mut apps,
            &mut inserter,
            &resolver,
            &foreground,
            title.clone(),
        )
        .await?;
        let current_usage = Usage {
            id: Default::default(),
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
            resolver,
        })
    }

    pub async fn handle(&mut self, event: Event) -> Result<()> {
        match event {
            Event::ForegroundChanged(ForegroundChangedEvent { at, window, title }) => {
                self.current_usage.end = at.into();
                self.inserter.insert_usage(&self.current_usage)?;

                let ws = WindowSession { window, title };
                info!("Foreground changed to {:?}", ws);
                let session_details = self
                    .sessions
                    .fallible_get_or_insert_async(ws.clone(), async {
                        Self::create_session_for_window(
                            &mut self.apps,
                            &mut self.inserter,
                            &self.resolver,
                            &ws.window,
                            ws.title,
                        )
                        .await
                    })
                    .await?;

                self.current_usage = Usage {
                    id: Default::default(),
                    session: session_details.session.clone(),
                    start: at.into(),
                    end: at.into(),
                };
            }
            Event::InteractionChanged(InteractionChangedEvent::BecameIdle {
                at,
                recorded_mouse_clicks,
                recorded_key_presses,
            }) => {
                info!("Became idle at {:?}", at);
                // TODO
            },
            Event::InteractionChanged(InteractionChangedEvent::BecameActive { at }) => {
                info!("Became active at {:?}", at);
                // TODO
            },
        };
        Ok(())
    }

    async fn create_app_for_process(
        inserter: &mut UsageWriter<'a>,
        resolver: &Sender<AppInfoResolverRequest>,
        pid: ProcessId,
        window: &Window,
    ) -> Result<AppDetails> {
        let process = Process::new(pid)?;

        let identity = if process.is_uwp(None)? {
            AppIdentity::Uwp {
                aumid: window.aumid()?,
            }
        } else {
            AppIdentity::Win32 {
                path: process.path()?,
            }
        };

        let found_app = inserter.find_or_insert_app(&identity)?;

        let app_id = match found_app {
            FoundOrInserted::Found(id) => id,
            FoundOrInserted::Inserted(id) => {
                resolver
                    .send_async(AppInfoResolverRequest::new(id.clone(), identity.clone()))
                    .await?;
                id
            }
        };
        Ok(AppDetails { app: app_id })
    }

    async fn create_session_for_window(
        apps: &mut HashMap<ProcessId, AppDetails>,
        inserter: &mut UsageWriter<'a>,
        resolver: &Sender<AppInfoResolverRequest>,
        window: &Window,
        title: String,
    ) -> Result<SessionDetails> {
        let pid = window.pid()?;
        let AppDetails { app } = apps
            .fallible_get_or_insert_async(pid, async {
                Self::create_app_for_process(inserter, resolver, pid, window).await
            })
            .await?;

        let mut session = Session {
            id: Default::default(),
            app: app.clone(),
            title,
        };
        inserter.insert_session(&mut session)?;

        Ok(SessionDetails {
            session: session.id,
        })
    }
}

trait HashMapExt<K, V> {
    async fn fallible_get_or_insert_async<'a>(
        &'a mut self,
        key: K,
        create: impl Future<Output = Result<V>>,
    ) -> Result<&'a mut V>
    where
        V: 'a;
}

impl<K: Eq + Hash + Clone, V> HashMapExt<K, V> for HashMap<K, V> {
    async fn fallible_get_or_insert_async<'a>(
        &'a mut self,
        key: K,
        create: impl Future<Output = Result<V>>,
    ) -> Result<&'a mut V>
    where
        V: 'a,
    {
        match self.entry(key.clone()) {
            Entry::Occupied(occ) => Ok(occ.into_mut()),
            Entry::Vacant(vac) => Ok(vac.insert(create.await?)),
        }
    }
}
