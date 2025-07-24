use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Arc;

use data::entities::{Alert, App, AppIdentity, Duration, Ref, Session, Timestamp};
use platform::events::{ForegroundWindowSessionInfo, WindowSession};
use platform::objects::{ProcessId, ProcessThreadId, Window};
use platform::web;
use scoped_futures::ScopedBoxFuture;
use util::ds::{SmallHashMap, SmallHashSet};
use util::error::Result;
use util::future as tokio;
use util::future::sync::RwLock;

/// Minimum opacity of a dimmed window
pub const MIN_DIM_LEVEL: f64 = 0.5;

/// Full opacity of a dimmed window
pub const FULL_DIM_LEVEL: f64 = 1.0;

/// Desktop State - for storing information about windows, processes, apps and sessions.
#[derive(Debug)]
pub struct DesktopStateInner {
    store: Store,
    web: WebsiteCache,
    platform: PlatformCache,
    actions: ActionStore,
}

/// Shared Desktop State
pub type DesktopState = Arc<RwLock<DesktopStateInner>>;

/// Create a new [DesktopState]
pub fn new_desktop_state(web_state: web::State) -> DesktopState {
    Arc::new(RwLock::new(DesktopStateInner::new(web_state)))
}

/// Cache for storing information about apps and sessions
#[derive(Debug)]
pub struct Store {
    // TODO this might be a bad idea, the HWND might be reused by Windows,
    // so another window could be running with the same HWND after the first one closed...
    sessions: SmallHashMap<WindowSession, SessionDetails>,
    // TODO this might be a bad idea, the pid/tid might be reused by Windows,
    // so another app could be running with the same pid/tid after the first one closed...
    apps: SmallHashMap<ProcessThreadId, AppDetails>,
}

/// Cache for storing information about websites and browsers
#[derive(Debug)]
pub struct WebsiteCache {
    // This never gets cleared, but it's ok since it's a small set of urls?
    websites: HashMap<web::BaseWebsiteUrl, AppDetails>,
    // This never gets cleared, but it's ok since it's a small set of apps?
    apps: HashMap<Ref<App>, web::BaseWebsiteUrl>,
    // Web state
    state: web::State,
}

/// Cache for storing information about windows and processes
#[derive(Debug)]
pub struct PlatformCache {
    // An app can have many processes open representing it.
    // A process can have many windows.
    windows: SmallHashMap<ProcessThreadId, HashSet<Window>>,
    processes: SmallHashMap<Ref<App>, AppEntry>,
}

/// Status of a dimmed window
#[derive(Debug, Default, Clone)]
pub struct DimStatus {
    requests: HashMap<Ref<Alert>, DimRequest>,
}

impl DimStatus {
    /// Create a new [DimStatus] with one request
    pub fn with_one(request: DimRequest) -> Self {
        Self {
            requests: HashMap::from([(request.by.clone(), request)]),
        }
    }

    /// Get the opacity of the dim
    pub fn opacity(&self, now: Timestamp) -> f64 {
        self.requests
            .values()
            .map(|r| r.opacity(now))
            .min_by(f64::total_cmp)
            .unwrap_or(FULL_DIM_LEVEL)
    }

    /// Add a dim request
    pub fn add(&mut self, request: DimRequest) {
        self.requests.insert(request.by.clone(), request);
    }

    /// Remove a dim request
    pub fn remove_by(&mut self, by: Ref<Alert>) {
        self.requests.remove(&by);
    }

    /// Subtract another [DimStatus] from this one
    pub fn subtract(&mut self, other: &DimStatus) {
        self.requests
            .retain(|_, r| !other.requests.contains_key(&r.by));
    }

    /// Merge another [DimStatus] into this one
    pub fn merge(&mut self, other: &DimStatus) {
        self.requests.extend(other.requests.clone());
    }

    /// Merge another [DimStatus] into this one
    pub fn into_merged(self, other: DimStatus) -> Self {
        let mut new = self;
        new.merge(&other);
        new
    }

    /// Check if there are any dim requests
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

/// Request to dim a window
#[derive(Debug, Clone)]
pub struct DimRequest {
    /// Who requested the dim
    pub by: Ref<Alert>,
    /// When the dim started
    pub start: Timestamp,
    /// Dim over this duration
    pub duration: Duration,
}

impl DimRequest {
    /// Get the opacity of the dim
    pub fn opacity(&self, now: Timestamp) -> f64 {
        let progress = (now - self.start) as f64 / (self.duration as f64);
        FULL_DIM_LEVEL - (progress.min(1.0) * (FULL_DIM_LEVEL - MIN_DIM_LEVEL))
    }
}

/// Store for storing actions for processes and websites
#[derive(Debug)]
pub struct ActionStore {
    /// Current dim status for each window. If not present, the window is not dimmed.
    pub dim: HashMap<Window, DimStatus>,
}

/// Details about a [App].
#[derive(Debug, Default)]
pub struct AppEntry {
    /// All [ProcessThreadId]s that are known to be a part of this app
    pub process_threads: SmallHashSet<ProcessThreadId>,
    /// Identity of the app
    pub identity: AppIdentity,
}

/// Details about a [Session].
#[derive(Debug)]
pub struct SessionDetails {
    /// Session Id
    pub session: Ref<Session>,
    /// Process Thread Id
    pub ptid: ProcessThreadId,
}

/// Details about a [App].
#[derive(Debug)]
pub struct AppDetails {
    /// App Id
    pub app: Ref<App>,
    /// Identity of the app
    pub identity: AppIdentity,
}

/// A process that can be killed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KillableProcessId {
    /// Win32 Process Id
    Win32(ProcessId),
    /// Aumid
    Aumid(String),
}

impl DesktopStateInner {
    /// Create a new [Cache].
    pub fn new(web_state: web::State) -> Self {
        Self {
            store: Store {
                sessions: HashMap::new(),
                apps: HashMap::new(),
            },
            web: WebsiteCache {
                websites: HashMap::new(),
                apps: HashMap::new(),
                state: web_state,
            },
            platform: PlatformCache {
                windows: HashMap::new(),
                processes: HashMap::new(),
            },
            actions: ActionStore {
                dim: HashMap::new(),
            },
        }
    }

    /// Get all processes for an [App]. Will return nothing for websites.
    pub fn platform_processes_for_app(&self, app: &Ref<App>) -> SmallHashSet<KillableProcessId> {
        let entry = self.platform.processes.get(app);
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
                _ => {
                    panic!(
                        "unsupported app identity for `platform_processes_for_app`: {:?}",
                        entry.identity
                    );
                }
            }
        } else {
            SmallHashSet::new()
        }
    }

    /// Get all windows for an [App]. Will return nothing for websites.
    pub fn platform_windows_for_app(&self, app: &Ref<App>) -> impl Iterator<Item = &Window> {
        self.platform
            .processes
            .get(app)
            .into_iter()
            .flat_map(|i| i.process_threads.iter())
            .flat_map(|ptid| {
                self.platform
                    .windows
                    .get(ptid)
                    .into_iter()
                    .flat_map(|windows| windows.iter())
            })
    }

    /// Get the websites for an [App]. If the app is not a website, will return nothing.
    pub fn websites_for_app(&self, app: &Ref<App>) -> impl Iterator<Item = &web::BaseWebsiteUrl> {
        self.web.apps.get(app).into_iter()
    }

    /// Get a mutable reference to the dim status for a [Window]
    pub fn dim_status_mut(&mut self, window: &Window) -> &mut DimStatus {
        self.actions.dim.entry(window.clone()).or_default()
    }

    /// Reset the dim statuses.
    /// The entries are not removed, but the requests are cleared.
    /// If the entries are still empty after this, the window will be undimmed
    /// and then this entry will be removed by Sentry.
    pub fn reset_dim_statuses(&mut self) {
        self.actions.dim.iter_mut().for_each(|(_, dim_status)| {
            dim_status.requests.clear();
        });
    }

    /// Get all dim statuses
    pub fn dim_statuses(&mut self) -> &mut HashMap<Window, DimStatus> {
        &mut self.actions.dim
    }

    /// Get or insert a [SessionDetails] for a [Window], using the create callback
    /// to make a new [SessionDetails] if not found.
    pub async fn get_or_insert_session_for_window<'a>(
        &'a mut self,
        session: ForegroundWindowSessionInfo,
        create: impl for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, Result<SessionDetails>>,
    ) -> Result<&'a mut SessionDetails> {
        // if-let doesn't work since the borrow lasts until end of function,
        // even if we return. Even if I surround this in another block.

        // if let Some(found) = self.sessions.get(&ws) {
        //     return Ok(found);
        // }
        if self.store.sessions.contains_key(&session.window_session) {
            return Ok(self
                .store
                .sessions
                .get_mut(&session.window_session)
                .unwrap());
        }

        let created = { create(self).await? };

        self.platform
            .windows
            .entry(created.ptid)
            .or_default()
            .insert(session.window_session.window.clone());

        Ok(self
            .store
            .sessions
            .entry(session.window_session)
            .or_insert(created))
    }

    /// Get or insert a [AppDetails] for a [ProcessThreadId], using the create callback
    /// to make a new [AppDetails] if not found.
    pub async fn get_or_insert_app_for_ptid<F: Future<Output = Result<AppDetails>>>(
        &mut self,
        ptid: ProcessThreadId,
        create: impl FnOnce(&mut Self) -> F,
    ) -> Result<&mut AppDetails> {
        if self.store.apps.contains_key(&ptid) {
            return Ok(self.store.apps.get_mut(&ptid).unwrap());
        }

        let created = { create(self).await? };
        let entry = self
            .platform
            .processes
            .entry(created.app.clone())
            .or_default();
        entry.process_threads.insert(ptid);
        entry.identity = created.identity.clone();

        Ok(self.store.apps.entry(ptid).or_insert(created))
    }

    /// Get or insert a [AppDetails] for a [web::BaseWebsiteUrl], using the create callback
    /// to make a new [AppDetails] if not found.
    pub async fn get_or_insert_website_for_base_url<F: Future<Output = Result<AppDetails>>>(
        &mut self,
        base_url: web::BaseWebsiteUrl,
        create: impl FnOnce(&mut Self) -> F,
    ) -> Result<&mut AppDetails> {
        if self.web.websites.contains_key(&base_url) {
            return Ok(self.web.websites.get_mut(&base_url).unwrap());
        }

        let created = { create(self).await? };
        self.web.apps.insert(created.app.clone(), base_url.clone());
        Ok(self.web.websites.entry(base_url).or_insert(created))
    }

    /// Remove a process and associated windows from the [Cache].
    pub async fn remove_process(&mut self, process: ProcessId) {
        let removed_windows = self
            .platform
            .windows
            .extract_if(|ptid, _| ptid.pid == process)
            .flat_map(|(_, windows)| windows)
            .collect::<SmallHashSet<_>>();

        self.platform.processes.retain(|_, entry| {
            // remove pid from list, and remove the entry altogether if it's empty
            entry.process_threads.retain(|ptid| ptid.pid != process);
            !entry.process_threads.is_empty()
        });

        self.store.apps.retain(|ptid, _| ptid.pid != process);
        self.store
            .sessions
            .retain(|ws, _| !removed_windows.contains(&ws.window));

        self.actions
            .dim
            .retain(|window, _| !removed_windows.contains(window));

        {
            let mut state = self.web.state.write().await;
            state.browser_processes.remove(&process);
            state
                .browser_windows
                .retain(|window, _| !removed_windows.contains(window));
        }
    }

    /// Remove an app and associated processes, windows from the [Cache].
    pub async fn remove_app(&mut self, app: Ref<App>) {
        let app_entry = self.platform.processes.remove(&app);
        if let Some(app_entry) = app_entry {
            let removed_windows = self
                .platform
                .windows
                .extract_if(|ptid, _| app_entry.process_threads.contains(ptid))
                .flat_map(|(_, windows)| windows)
                .collect::<SmallHashSet<_>>();

            self.store
                .apps
                .retain(|ptid, _| !app_entry.process_threads.contains(ptid));
            self.store
                .sessions
                .retain(|ws, _| !removed_windows.contains(&ws.window));

            self.actions
                .dim
                .retain(|window, _| !removed_windows.contains(window));

            let removed_pids = app_entry
                .process_threads
                .iter()
                .map(|ptid| ptid.pid)
                .collect::<SmallHashSet<_>>();
            {
                let mut state = self.web.state.write().await;
                state
                    .browser_processes
                    .retain(|pid| !removed_pids.contains(pid));
                state
                    .browser_windows
                    .retain(|window, _| !removed_windows.contains(window));
            }
        }
    }

    /// Retains all process for windows in the list, and removes the rest
    /// of the process and windows not in the list.
    pub async fn retain_cache(&mut self) -> Result<()> {
        let mut removed_windows = SmallHashSet::new();
        self.platform.windows.retain(|ptid, windows| {
            // check if window is alive by checking if the pid() calls
            // still succeeds and returns the same pid as previously
            // returned to the engine
            removed_windows.extend(windows.extract_if(|window| window.ptid().ok() != Some(*ptid)));
            // remove entry if it's empty
            !windows.is_empty()
        });

        // retain only app refs for processes that are alive
        let mut removed_pids = SmallHashSet::new();
        self.platform.processes.retain(|_, entry| {
            removed_pids.extend(
                entry
                    .process_threads
                    .extract_if(|ptid| !self.platform.windows.contains_key(ptid))
                    .map(|ptid| ptid.pid),
            );
            !entry.process_threads.is_empty()
        });

        // retain only sessions and apps for which their windows and apps that are alive
        self.store
            .sessions
            .retain(|ws, _| !removed_windows.contains(&ws.window));
        self.store
            .apps
            .retain(|ptid, _| !removed_pids.contains(&ptid.pid));

        self.actions
            .dim
            .retain(|window, _| !removed_windows.contains(window));

        {
            let mut state = self.web.state.write().await;
            state
                .browser_processes
                .retain(|process| !removed_pids.contains(process));
            state
                .browser_windows
                .retain(|window, _| !removed_windows.contains(window));
        }
        Ok(())
    }
}

#[tokio::test]
async fn inner_mut_compiles() {
    use scoped_futures::ScopedFutureExt;

    let window: Window = Window::foreground().unwrap();
    let process = ProcessThreadId { pid: 1, tid: 1 };
    let web_state = web::default_state();
    let mut desktop = DesktopStateInner::new(web_state);

    desktop
        .get_or_insert_session_for_window(
            ForegroundWindowSessionInfo {
                window_session: WindowSession {
                    window,
                    title: "".to_string(),
                    url: None,
                },
                fetched_path: None,
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
                            })
                        })
                        .await?;
                    Ok(SessionDetails {
                        session: Ref::new(1),
                        ptid: process,
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
