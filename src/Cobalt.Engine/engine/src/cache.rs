use std::collections::HashMap;

use data::entities::{App, Ref, Session};
use platform::objects::{ProcessId, Window};
use util::error::Result;

// Everyone holds a Rc<Refcell<Cache>>, presumably.

pub struct Cache {
    // TODO UsageWriter, Spawner, Config, or maybe just the generate functions?

    // TODO this might be a bad idea, the HWND might be reused by Windows,
    // so another window could be running with the same HWND after the first one closed...
    sessions: HashMap<WindowSession, SessionDetails>,
    // TODO this might be a bad idea, the ProcessId might be reused by Windows,
    // so another app could be running with the same pid after the first one closed...
    apps: HashMap<ProcessId, AppDetails>,

    // An app can have many processes open representing it.
    // A process can have many windows.
    windows: HashMap<ProcessId, Vec<Window>>,
    processes: HashMap<Ref<App>, Vec<ProcessId>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    pub window: Window,
    pub title: String,
}

pub struct SessionDetails {
    pub session: Ref<Session>,
}

pub struct AppDetails {
    pub app: Ref<App>,
}

// TODO transfer everything from engine into here.

impl Cache {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            apps: HashMap::new(),
            windows: HashMap::new(),
            processes: HashMap::new(),
        }
    }

    pub fn get_or_insert_session_for_window(
        &mut self,
        ws: WindowSession,
        create: impl FnOnce(&mut Self) -> Result<SessionDetails>,
    ) -> Result<&mut SessionDetails> {
        // if-let doesn't work since the borrow lasts until end of function,
        // even if we return. Even if I surround this in another block.

        // if let Some(found) = self.sessions.get(&ws) {
        //     return Ok(found);
        // }
        if self.sessions.contains_key(&ws) {
            return Ok(self.sessions.get_mut(&ws).unwrap());
        }

        let created = { create(self)? };
        self.windows
            .entry(ws.window.pid()?)
            .or_insert_with(|| vec![])
            .push(ws.window.clone());

        Ok(self.sessions.entry(ws).or_insert(created))
    }

    pub fn get_or_insert_app_for_process(
        &mut self,
        process: ProcessId,
        create: impl FnOnce(&mut Self) -> Result<AppDetails>,
    ) -> Result<&mut AppDetails> {
        if self.apps.contains_key(&process) {
            return Ok(self.apps.get_mut(&process).unwrap());
        }

        let created = { create(self)? };
        self.processes
            .entry(created.app.clone())
            .or_insert_with(|| vec![])
            .push(process);

        Ok(self.apps.entry(process).or_insert(created))
    }

    // pub async fn get_or_insert_session_for_window(&mut self, window: Window, create: impl Future<Output = Result<SessionDetails>>) -> Result<&SessionDetails> {

    //     unimplemented!()
    // }

    // pub async fn get_or_insert_app_for_process(&mut self, process: Process, create: impl Future<Output = Result<AppDetails>>) -> Result<&AppDetails> {
    //     unimplemented!()
    // }
}

#[test]
fn inner_mut_compiles() {
    let window: Window = Window::foreground().unwrap();
    let process: ProcessId = 1;
    let mut cache = Cache::new();

    cache
        .get_or_insert_session_for_window(
            WindowSession {
                window,
                title: "".to_string(),
            },
            |cache| {
                let _app = cache.get_or_insert_app_for_process(process, |_| {
                    Ok(AppDetails { app: Ref::new(1) })
                })?;
                Ok(SessionDetails {
                    session: Ref::new(1),
                })
            },
        )
        .unwrap();
    // cache.get_or_insert_session_for_window(window, async {
    //     let app = cache.get_or_insert_app_for_process(process, async {
    //         Ok(AppDetails {
    //             app: Ref::new(1),
    //         })
    //     }).await?;
    //     Ok(SessionDetails {
    //         session: Ref::new(1),
    //     })
    // }).await.unwrap();
}
