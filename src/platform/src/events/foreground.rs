use std::time::Duration;

use util::config::Config;
use util::error::{Context, Result, eyre};
use util::tracing::ResultTraceExt;

use crate::objects::{Process, Timestamp, Window};
use crate::web::{BrowserDetector, BrowserWindowInfo, State, WriteLockedState};

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
            let (browser_window_info, fetched_path) = {
                let browser_window_info = browser_state.browser_windows.get(&fg).cloned();
                if let Some(browser_window_info) = browser_window_info {
                    // We know already if it's a browser or not
                    (browser_window_info, None)
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

                    let browser_window_info = if is_browser {
                        let element = browser.get_chromium_element(&fg)?;

                        // Loop until we get a valid incognito state. Invalid states are when the broser is loading
                        // from memory save, for example - the browser is not fully loaded at that time. Max timeout is 3 seconds.
                        let is_incognito = loop_until_valid(
                            || {
                                let incognito = browser
                                    .chromium_incognito(&element)
                                    .context("get chromium incognito")?;

                                Ok(incognito)
                            },
                            Duration::from_millis(100),
                            Duration::from_secs(3),
                        )?;

                        let url = if is_incognito && !track_incognito {
                            None
                        } else {
                            let element = browser.get_chromium_element(&fg)?;
                            let url = browser
                                .chromium_url(&element, None)
                                .context("get chromium url")?;
                            url
                        };

                        Some(BrowserWindowInfo { is_incognito, url })
                    } else {
                        None
                    };

                    browser_state
                        .browser_windows
                        .insert(fg.clone(), browser_window_info.clone());

                    (browser_window_info, fetched_path)
                }
            };
            drop(browser_state);

            let (title, url) = if let Some(browser_window_info) = browser_window_info {
                if browser_window_info.is_incognito && !track_incognito {
                    ("<Incognito>".to_string(), None)
                } else {
                    let title = fg.title()?;
                    (title, browser_window_info.url)
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

fn loop_until_valid<T>(
    f: impl Fn() -> Result<Option<T>>,
    sleep_duration: Duration,
    timeout: Duration,
) -> Result<T> {
    let start = std::time::Instant::now();
    loop {
        let result = f()?;
        if let Some(result) = result {
            return Ok(result);
        }
        if start.elapsed() > timeout {
            return Err(eyre!("Timeout while waiting for valid result"));
        }
        std::thread::sleep(sleep_duration);
    }
}
