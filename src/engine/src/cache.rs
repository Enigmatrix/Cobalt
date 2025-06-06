use std::collections::{HashMap, HashSet};
use std::future::Future;

use data::entities::{App, AppIdentity, Ref, Session};
use platform::events::WindowSession;
use platform::objects::{BaseWebsiteUrl, ProcessId, ProcessThreadId, Window};
use scoped_futures::ScopedBoxFuture;
use util::error::Result;
use util::future as tokio;

/// Cache for storing information about windows, processes, apps and sessions.
#[derive(Debug)]
pub struct Cache {
    // TODO this might be a bad idea, the HWND might be reused by Windows,
    // so another window could be running with the same HWND after the first one closed...
    sessions: HashMap<WindowSession, SessionDetails>,
    // TODO this might be a bad idea, the pid/tid might be reused by Windows,
    // so another app could be running with the same pid/tid after the first one closed...
    apps: HashMap<ProcessThreadId, AppDetails>,

    // This never gets cleared, but it's ok since it's a small set of urls?
    websites: HashMap<BaseWebsiteUrl, AppDetails>,
    // Cache of whether a window is a browser or not.
    browsers: HashMap<Window, bool>,

    // An app can have many processes open representing it.
    // A process can have many windows.
    windows: HashMap<ProcessThreadId, HashSet<Window>>,
    processes: HashMap<Ref<App>, AppEntry>,
}

/// Details about a [App].
#[derive(Debug, Default)]
pub struct AppEntry {
    pub process_threads: HashSet<ProcessThreadId>,
    pub identity: AppIdentity,
}

/// Details about a [Session].
#[derive(Debug)]
pub struct SessionDetails {
    pub session: Ref<Session>,
    pub ptid: ProcessThreadId,
    pub is_browser: bool,
}

/// Details about a [App].
#[derive(Debug)]
pub struct AppDetails {
    pub app: Ref<App>,
    pub identity: AppIdentity,
    pub is_browser: bool,
}

/// A process that can be killed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KillableProcessId {
    Win32(ProcessId),
    Aumid(String),
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
    pub fn processes_for_app(&self, app: &Ref<App>) -> HashSet<KillableProcessId> {
        let entry = self.processes.get(app);
        if let Some(entry) = entry {
            match &entry.identity {
                AppIdentity::Uwp { aumid } => [KillableProcessId::Aumid(aumid.clone())]
                    .into_iter()
                    .collect(),
                AppIdentity::Win32 { .. } => entry
                    .process_threads
                    .iter()
                    .map(|ptid| KillableProcessId::Win32(ptid.pid))
                    .collect(),
                // TODO handle Website entries
                _ => {
                    todo!()
                }
            }
        } else {
            HashSet::new()
        }
    }

    /// Get all windows for an [App].
    pub fn windows_for_app(&self, app: &Ref<App>) -> impl Iterator<Item = &Window> {
        self.processes
            .get(app)
            .into_iter()
            .flat_map(|i| i.process_threads.iter())
            .flat_map(|ptid| {
                self.windows
                    .get(ptid)
                    .into_iter()
                    .flat_map(|windows| windows.iter())
            })
    }

    /// Remove a process and associated windows from the [Cache].
    pub fn remove_process(&mut self, process: ProcessId) {
        self.apps.retain(|ptid, _| ptid.pid != process);
        let removed_windows = self
            .windows
            .iter()
            .filter(|(ptid, _)| ptid.pid == process)
            .flat_map(|(_, windows)| windows)
            .collect::<HashSet<_>>();
        self.sessions
            .retain(|ws, _| !removed_windows.contains(&ws.window));
        self.browsers
            .retain(|window, _| !removed_windows.contains(window));
        self.windows.retain(|ptid, _| ptid.pid != process);
        self.processes.retain(|_, entry| {
            // remove pid from list, and remove the entry altogether if it's empty
            entry.process_threads.retain(|ptid| ptid.pid != process);
            !entry.process_threads.is_empty()
        });
    }

    /// Remove an app and associated processes, windows from the [Cache].
    pub fn remove_app(&mut self, app: Ref<App>) {
        let app_entry = self.processes.remove(&app);
        if let Some(app_entry) = app_entry {
            for ptid in &app_entry.process_threads {
                self.windows.remove(ptid);
                self.apps.remove(ptid);
            }

            let removed_windows = self
                .windows
                .iter()
                .filter(|(ptid, _)| app_entry.process_threads.contains(ptid))
                .flat_map(|(_, windows)| windows)
                .collect::<HashSet<_>>();
            self.sessions
                .retain(|ws, _| !removed_windows.contains(&ws.window));
            self.browsers
                .retain(|window, _| !removed_windows.contains(window));
        }
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
            .entry(created.ptid)
            .or_default()
            .insert(ws.window.clone());

        Ok(self.sessions.entry(ws).or_insert(created))
    }

    /// Get or insert a [AppDetails] for a [ProcessThreadId], using the create callback
    /// to make a new [AppDetails] if not found.
    pub async fn get_or_insert_app_for_ptid<F: Future<Output = Result<AppDetails>>>(
        &mut self,
        ptid: ProcessThreadId,
        create: impl FnOnce(&mut Self) -> F,
    ) -> Result<&mut AppDetails> {
        if self.apps.contains_key(&ptid) {
            return Ok(self.apps.get_mut(&ptid).unwrap());
        }

        let created = { create(self).await? };
        let entry = self.processes.entry(created.app.clone()).or_default();
        entry.process_threads.insert(ptid);
        entry.identity = created.identity.clone();

        Ok(self.apps.entry(ptid).or_insert(created))
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
        self.windows.retain(|ptid, windows| {
            // check if window is alive by checking if the pid() calls
            // still succeeds and returns the same pid as previously
            // returned to the engine
            windows.retain(|window| window.ptid().ok() == Some(*ptid));
            !windows.is_empty()
        });
        let alive_windows = self.windows.values().flatten().collect::<HashSet<_>>();

        // retain only sessions and apps for which their windows and apps that are alive
        self.sessions
            .retain(|ws, _| alive_windows.contains(&ws.window));
        self.apps.retain(|ptid, _| self.windows.contains_key(ptid));
        // retain only app refs for processes that are alive
        self.processes.retain(|_, entry| {
            entry
                .process_threads
                .retain(|ptid| self.windows.contains_key(ptid));
            !entry.process_threads.is_empty()
        });
        Ok(())
    }
}

#[tokio::test]
async fn inner_mut_compiles() {
    use scoped_futures::ScopedFutureExt;

    let window: Window = Window::foreground().unwrap();
    let process = ProcessThreadId { pid: 1, tid: 1 };
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
                        .get_or_insert_app_for_ptid(process, |_| async {
                            Ok(AppDetails {
                                app: Ref::new(1),
                                identity: AppIdentity::Win32 {
                                    path: "C:\\yorm.exe".to_string(),
                                },
                                is_browser: true,
                            })
                        })
                        .await?;
                    Ok(SessionDetails {
                        session: Ref::new(1),
                        ptid: process,
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
