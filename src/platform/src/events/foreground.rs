use util::config::Config;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use crate::objects::{Process, Timestamp, Window};
use crate::web;

/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
    detect: web::Detect,
    track_incognito: bool,
    web_state: web::State,
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
    pub fn new(session: WindowSession, config: &Config, web_state: web::State) -> Result<Self> {
        let detect = web::Detect::new()?;
        Ok(Self {
            session,
            detect,
            track_incognito: config.track_incognito(),
            web_state,
        })
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        let state = self.web_state.blocking_write();
        if let Some(session) =
            Self::foreground_window_session(&self.detect, state, self.track_incognito)?
        {
            if session == self.session {
                return Ok(None);
            }
            self.session = session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }

    /// Get the foreground window session.
    pub fn foreground_window_session(
        detect: &web::Detect,
        web_state: web::WriteLockedState<'_>,
        track_incognito: bool,
    ) -> Result<Option<WindowSession>> {
        if let Some(window) = Window::foreground() {
            let (is_browser, fetched_path) =
                Self::browser_window_session(&window, detect, web_state)?;

            let (title, url) = if is_browser {
                let element = detect.get_chromium_element(&window)?;
                let incognito = detect
                    .chromium_incognito(&element)
                    .context("get chromium incognito")?;
                if let Some(incognito) = incognito
                    && incognito
                    && !track_incognito
                {
                    ("<Incognito>".to_string(), None)
                } else {
                    let url = detect.chromium_url(&element).context("get chromium url")?;
                    let title = window.title()?;
                    (title, url)
                }
            } else {
                (window.title()?, None)
            };

            let session = WindowSession {
                title,
                url,
                window,
                fetched_path,
            };
            return Ok(Some(session));
        }
        Ok(None)
    }

    fn browser_window_session(
        window: &Window,
        detect: &web::Detect,
        mut web_state: web::WriteLockedState<'_>,
    ) -> Result<(bool, Option<String>)> {
        let is_browser = web_state.browser_windows.get(window).cloned();
        if let Some(is_browser) = is_browser {
            // We know already if it's a browser or not
            return Ok((is_browser, None));
        }

        // Educated guess. Note that VSCode will pass this check since it's a chromium app.
        let maybe_browser = detect.is_maybe_chromium_window(window).warn();
        // So we further check if it's a chromium process
        let (is_browser, fetched_path) = if maybe_browser {
            let ptid = window.ptid().context("get ptid")?;
            if web_state.browser_processes.contains(&ptid.pid) {
                (true, None)
            } else {
                let process = Process::new(ptid.pid).context("get process")?;
                let path = process.path().context("get process path")?;
                let is_browser = detect.is_chromium_exe(&path);
                if is_browser {
                    web_state.browser_processes.insert(ptid.pid);
                }
                (is_browser, Some(path))
            }
        } else {
            (false, None)
        };

        web_state.browser_windows.insert(window.clone(), is_browser);

        Ok((is_browser, fetched_path))
    }
}
