use std::collections::hash_map::Entry;
use std::collections::HashMap;

use data::db::{FoundOrInserted, UsageWriter};
use data::entities::{App, AppIdentity, Ref, Session, Usage};
use platform::events::{ForegroundChangedEvent, InteractionChangedEvent};
use platform::objects::{Process, ProcessId, Window};
use std::hash::Hash;
use util::channels::Sender;
use util::error::Result;

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
    pub fn handle(&mut self, event: Event) -> Result<()> {
        match event {
            Event::ForegroundChanged(ForegroundChangedEvent { at, window, title }) => {
                self.current_usage.end = at.ticks();
                self.inserter.insert_usage(&self.current_usage)?;

                let session_details = self.sessions.fallible_get_or_insert(
                    WindowSession { window, title },
                    |ws| {
                        Self::create_session_for_window(
                            &mut self.apps,
                            &mut self.inserter,
                            &self.resolver,
                            &ws.window,
                            ws.title,
                        )
                    },
                )?;

                self.current_usage = Usage {
                    id: Default::default(),
                    session: session_details.session.clone(),
                    start: at.ticks(),
                    end: at.ticks(),
                };
            }
            Event::InteractionChanged(InteractionChangedEvent::BecameIdle {
                at,
                recorded_mouse_clicks,
                recorded_key_presses,
            }) => todo!(),
            Event::InteractionChanged(InteractionChangedEvent::BecameActive { at }) => todo!(),
        };
        Ok(())
    }

    fn create_app_for_process(
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
                resolver.send(AppInfoResolverRequest::new(id.clone(), identity.clone()))?;
                id
            }
        };
        Ok({ AppDetails { app: app_id } })
    }

    fn create_session_for_window(
        apps: &mut HashMap<ProcessId, AppDetails>,
        inserter: &mut UsageWriter<'a>,
        app_info_tx: &Sender<AppInfoResolverRequest>,
        window: &Window,
        title: String,
    ) -> Result<SessionDetails> {
        let pid = window.pid()?;
        let AppDetails { app } = apps.fallible_get_or_insert(pid, |pid| {
            Self::create_app_for_process(inserter, app_info_tx, pid, window)
        })?;

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
            Entry::Vacant(vac) => Ok(vac.insert(create(key)?)),
        }
    }
}
