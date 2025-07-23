use util::config::Config;
use util::error::{Context, ContextCompat, Result};
use util::tracing::ResultTraceExt;
use windows::core::AgileReference;

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
    /// New [ForegroundWindowSessionInfo] details
    pub session: ForegroundWindowSessionInfo,
}

/// A session of [Window] with additional info
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ForegroundWindowSessionInfo {
    /// Foreground [WindowSession]
    pub window_session: WindowSession,
    /// Path to the executable, if fetched
    pub fetched_path: Option<String>,
}

/// A session of [Window]. Each session has a unique title, url.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    /// Foreground [Window]
    pub window: Window,
    /// Foreground [Window] title
    pub title: String,
    /// Foreground [Window] URL, if Browser
    pub url: Option<String>,
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
            if session.window_session == self.session {
                return Ok(None);
            }
            self.session = session.window_session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }

    /// Get the foreground window session.
    pub fn foreground_window_session(
        detect: &web::Detect,
        web_state: web::WriteLockedState<'_>,
        track_incognito: bool,
    ) -> Result<Option<ForegroundWindowSessionInfo>> {
        if let Some(window) = Window::foreground() {
            let (state, fetched_path) = Self::browser_window_session(&window, detect, web_state)?;

            let (title, url) = if let Some(state) = state {
                if state.is_incognito && !track_incognito {
                    ("<Incognito>".to_string(), None)
                } else {
                    (state.last_title, Some(state.last_url))
                }
            } else {
                (window.title()?, None)
            };

            let window_session = WindowSession { title, url, window };
            let session = ForegroundWindowSessionInfo {
                window_session,
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
    ) -> Result<(Option<web::BrowserWindowState>, Option<String>)> {
        let state = web_state.browser_windows.get(window);
        if let Some(state) = state {
            // We know already if it's a browser or not
            return Ok((state.clone(), None));
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

        // TODO: instead of Option's context("no element") we should use backon retries
        let state = if is_browser {
            let window_element = detect.get_chromium_element(window)?;

            let omnibox = detect
                .get_chromium_omnibox_element(&window_element, true)?
                .context("no omnibox")?;
            let omnibox_icon = detect
                .get_chromium_omnibox_icon_element(&window_element, true)?
                .context("no omnibox icon")?;
            // this is the element that is least likely to exist since the page could not be loaded
            let root_web_area = detect
                .get_chromium_root_web_area_element(&window_element, true)?
                .context("no root web area")?;

            let is_incognito = detect
                .chromium_incognito(&window_element)
                .context("get chromium incognito")?
                .context("no element")?;
            let last_url = detect
                .chromium_url(&window_element)
                .context("get chromium url")?
                .context("no element")?;
            let last_title = window.title()?;

            let extracted_elements = web::ExtractedUIElements {
                window_element: AgileReference::new(&window_element)?,
                omnibox: AgileReference::new(&omnibox)?,
                omnibox_icon: AgileReference::new(&omnibox_icon)?,
                root_web_area: AgileReference::new(&root_web_area)?,
            };
            Some(web::BrowserWindowState {
                extracted_elements,
                is_incognito,
                last_url,
                last_title,
            })
        } else {
            None
        };

        web_state
            .browser_windows
            .insert(window.clone(), state.clone());

        Ok((state, fetched_path))
    }
}
