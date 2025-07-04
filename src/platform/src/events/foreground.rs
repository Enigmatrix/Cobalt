use util::config::Config;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use crate::objects::{Process, Timestamp, Window};
use crate::web::{BrowserDetector, State, WriteLockedState};

/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
    browser: BrowserDetector,
    track_incognito: bool,
    browser_state: State,
}

/// Foreground window session change event.
pub struct ForegroundChangedEvent {
    /// Timestamp of foreground window session change
    pub at: Timestamp,
    /// New [WindowSession] details
    pub session: WindowSession,
}

/// A session of [Window]. Each session has a unique title.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    /// Foreground [Window]
    pub window: Window,
    /// Foreground [Window] title
    pub title: String,
    /// Foreground [Window] URL, if Browser
    pub url: Option<String>,
    /// Path to the executable, if fetched
    pub fetched_path: Option<String>,
}

impl ForegroundEventWatcher {
    /// Create a new [WindowSession] with the starting [WindowSession].
    pub fn new(session: WindowSession, config: &Config, browser_state: State) -> Result<Self> {
        let browser = BrowserDetector::new()?;
        Ok(Self {
            session,
            browser,
            track_incognito: config.track_incognito(),
            browser_state,
        })
    }

    /// Get the foreground window session.
    pub fn foreground_window_session(
        browser: &BrowserDetector,
        mut browser_state: WriteLockedState<'_>,
        track_incognito: bool,
    ) -> Result<Option<WindowSession>> {
        if let Some(fg) = Window::foreground() {
            let (is_browser, fetched_path) = {
                let is_browser = browser_state.browser_windows.get(&fg).cloned();
                if let Some(is_browser) = is_browser {
                    // We know already if it's a browser or not
                    (is_browser, None)
                } else {
                    // Educated guess. Note that VSCode will pass this check since it's a chromium app.
                    let maybe_browser = browser.is_maybe_chromium_window(&fg).warn();
                    // So we further check if it's a chromium process
                    let (is_browser, fetched_path) = if maybe_browser {
                        let ptid = fg.ptid().context("get ptid")?;
                        if browser_state.browser_processes.contains(&ptid.pid) {
                            (true, None)
                        } else {
                            let process = Process::new(ptid.pid).context("get process")?;
                            let path = process.path().context("get process path")?;
                            let is_browser = BrowserDetector::is_maybe_chromium_exe(&path);
                            if is_browser {
                                browser_state.browser_processes.insert(ptid.pid);
                            }
                            (is_browser, Some(path))
                        }
                    } else {
                        (false, None)
                    };

                    browser_state.browser_windows.insert(fg.clone(), is_browser);

                    (is_browser, fetched_path)
                }
            };
            drop(browser_state);

            let (title, url) = if is_browser {
                let element = browser.get_chromium_element(&fg)?;
                let incognito = browser
                    .chromium_incognito(&element)
                    .context("get chromium incognito")?;
                if let Some(incognito) = incognito
                    && incognito
                    && !track_incognito
                {
                    ("<Incognito>".to_string(), None)
                } else {
                    let url = browser.chromium_url(&element).context("get chromium url")?;
                    let title = fg.title()?;
                    (title, url)
                }
            } else {
                (fg.title()?, None)
            };

            let session = WindowSession {
                title,
                url,
                window: fg,
                fetched_path,
            };
            return Ok(Some(session));
        }
        Ok(None)
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        let state = self.browser_state.blocking_write();
        if let Some(session) =
            Self::foreground_window_session(&self.browser, state, self.track_incognito)?
        {
            if session == self.session {
                return Ok(None);
            }
            self.session = session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }
}
