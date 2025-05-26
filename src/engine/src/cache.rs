use std::collections::{HashMap, HashSet};
use std::future::Future;

use data::entities::{App, Ref, Session};
use platform::events::WindowSession;
use platform::objects::{BaseWebsiteUrl, ProcessId, Window};
use scoped_futures::ScopedBoxFuture;
use util::error::Result;
use util::future as tokio;

/// Cache for storing information about windows, processes, apps and sessions.
#[derive(Debug)]
pub struct Cache {
    // TODO this might be a bad idea, the HWND might be reused by Windows,
    // so another window could be running with the same HWND after the first one closed...
    sessions: HashMap<WindowSession, SessionDetails>,
    // TODO this might be a bad idea, the ProcessId might be reused by Windows,
    // so another app could be running with the same pid after the first one closed...
    apps: HashMap<ProcessId, AppDetails>,

    // This never gets cleared, but it's ok since it's a small set of urls?
    websites: HashMap<BaseWebsiteUrl, AppDetails>,
    // Cache of whether a window is a browser or not.
    browsers: HashMap<Window, bool>,

    // An app can have many processes open representing it.
    // A process can have many windows.
    windows: HashMap<ProcessId, HashSet<Window>>,
    processes: HashMap<Ref<App>, HashSet<ProcessId>>,
}

/// Details about a [Session].
#[derive(Debug)]
pub struct SessionDetails {
    pub session: Ref<Session>,
    pub pid: ProcessId,
    pub is_browser: bool,
}

/// Details about a [App].
#[derive(Debug)]
pub struct AppDetails {
    pub app: Ref<App>,
    pub is_browser: bool,
}

impl Cache {
    /// Create a new [Cache].
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            apps: HashMap::new(),
            websites: HashMap::new(),
            browsers: HashMap::new(),
            windows: HashMap::new(),
            processes: HashMap::new(),
        }
    }

    /// Get all processes for an [App].
    pub fn processes_for_app(&self, app: &Ref<App>) -> impl Iterator<Item = &ProcessId> {
        self.processes.get(app).into_iter().flat_map(|i| i.iter())
    }

    /// Get all windows for an [App].
    pub fn windows_for_app(&self, app: &Ref<App>) -> impl Iterator<Item = &Window> {
        self.processes_for_app(app).flat_map(|pid| {
            self.windows
                .get(pid)
                .into_iter()
                .flat_map(|windows| windows.iter())
        })
    }

    /// Remove a process and associated windows from the [Cache].
    pub fn remove_process(&mut self, process: ProcessId) {
        self.apps.remove(&process);
        if let Some(windows) = self.windows.remove(&process) {
            self.sessions.retain(|ws, _| !windows.contains(&ws.window));
            self.browsers.retain(|window, _| !windows.contains(window));
        }
        self.processes.retain(|_, pids| {
            // remove pid from list, and remove the entry altogether if it's empty
            pids.retain(|pid| *pid != process);
            !pids.is_empty()
        });
    }

    /// Get or insert a [SessionDetails] for a [Window], using the create callback
    /// to make a new [SessionDetails] if not found.
    pub async fn get_or_insert_session_for_window<'a>(
        &'a mut self,
        mut ws: WindowSession,
        create: impl for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, Result<SessionDetails>>,
    ) -> Result<&'a mut SessionDetails> {
        // if-let doesn't work since the borrow lasts until end of function,
        // even if we return. Even if I surround this in another block.

        // if let Some(found) = self.sessions.get(&ws) {
        //     return Ok(found);
        // }
        if self.sessions.contains_key(&ws) {
            return Ok(self.sessions.get_mut(&ws).unwrap());
        }

        let created = { create(self).await? };

        if !created.is_browser {
            ws.url = None;
        }
        self.browsers.insert(ws.window.clone(), created.is_browser);

        self.windows
            .entry(created.pid)
            .or_default()
            .insert(ws.window.clone());

        Ok(self.sessions.entry(ws).or_insert(created))
    }

    /// Get or insert a [AppDetails] for a [ProcessId], using the create callback
    /// to make a new [AppDetails] if not found.
    pub async fn get_or_insert_app_for_process<F: Future<Output = Result<AppDetails>>>(
        &mut self,
        process: ProcessId,
        create: impl FnOnce(&mut Self) -> F,
    ) -> Result<&mut AppDetails> {
        if self.apps.contains_key(&process) {
            return Ok(self.apps.get_mut(&process).unwrap());
        }

        let created = { create(self).await? };
        self.processes
            .entry(created.app.clone())
            .or_default()
            .insert(process);

        Ok(self.apps.entry(process).or_insert(created))
    }

    /// Get or insert a [AppDetails] for a [BaseWebsiteUrl], using the create callback
    /// to make a new [AppDetails] if not found.
    pub async fn get_or_insert_website_for_base_url<F: Future<Output = Result<AppDetails>>>(
        &mut self,
        base_url: BaseWebsiteUrl,
        create: impl FnOnce(&mut Self) -> F,
    ) -> Result<&mut AppDetails> {
        if self.websites.contains_key(&base_url) {
            return Ok(self.websites.get_mut(&base_url).unwrap());
        }

        let created = { create(self).await? };
        Ok(self.websites.entry(base_url).or_insert(created))
    }

    pub fn is_browser(&self, window: &Window) -> Option<bool> {
        self.browsers.get(window).copied()
    }

    // pub async fn get_or_insert_session_for_window(&mut self, window: Window, create: impl Future<Output = Result<SessionDetails>>) -> Result<&SessionDetails> {

    //     unimplemented!()
    // }

    // pub async fn get_or_insert_app_for_process(&mut self, process: Process, create: impl Future<Output = Result<AppDetails>>) -> Result<&AppDetails> {
    //     unimplemented!()
    // }

    /// Retains all process for windows in the list, and removes the rest
    /// of the process and windows not in the list.
    pub fn retain_cache(&mut self) -> Result<()> {
        self.windows.retain(|pid, windows| {
            // check if window is alive by checking if the pid() calls
            // still succeeds and returns the same pid as previously
            // returned to the engine
            windows.retain(|window| window.pid().ok() == Some(*pid));
            !windows.is_empty()
        });
        let alive_windows = self.windows.values().flatten().collect::<HashSet<_>>();

        // retain only sessions and apps for which their windows and apps that are alive
        self.sessions
            .retain(|ws, _| alive_windows.contains(&ws.window));
        self.apps.retain(|pid, _| self.windows.contains_key(pid));
        // retain only app refs for procesess that are alive
        self.processes.retain(|_, pids| {
            pids.retain(|pid| self.windows.contains_key(pid));
            !pids.is_empty()
        });
        Ok(())
    }
}

#[tokio::test]
async fn inner_mut_compiles() {
    use scoped_futures::ScopedFutureExt;

    let window: Window = Window::foreground().unwrap();
    let process: ProcessId = 1;
    let mut cache = Cache::new();

    cache
        .get_or_insert_session_for_window(
            WindowSession {
                window,
                title: "".to_string(),
                url: None,
            },
            |cache| {
                async move {
                    let _app = cache
                        .get_or_insert_app_for_process(process, |_| async {
                            Ok(AppDetails {
                                app: Ref::new(1),
                                is_browser: true,
                            })
                        })
                        .await?;
                    Ok(SessionDetails {
                        session: Ref::new(1),
                        pid: process,
                        is_browser: true,
                    })
                }
                .scope_boxed()
            },
        )
        .await
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
