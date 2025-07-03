use util::config::Config;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use crate::objects::{Timestamp, Window};
use crate::web::{BrowserDetector, State, StateInner};

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
        browser_state: &StateInner,
        track_incognito: bool,
    ) -> Result<Option<WindowSession>> {
        if let Some(fg) = Window::foreground() {
            let maybe_browser = {
                let is_browser = browser_state.browser_windows.get(&fg).cloned();
                // Either we definitely know it's a browser, or we try to make an educated guess using browser.is_chromium
                if let Some(is_browser) = is_browser {
                    is_browser
                } else {
                    // Educated guess. Note that VSCode will pass this check since it's a chromium app.
                    // However, we shouldn't arrive here more than once since we're using the cache.
                    browser.is_maybe_chromium_window(&fg).warn()
                }
            };

            let (title, url) = if maybe_browser {
                let url = browser.chromium_url(&fg).context("get chromium url")?;
                if url.incognito && !track_incognito {
                    ("<Incognito>".to_string(), None)
                } else {
                    (fg.title()?, url.url)
                }
            } else {
                (fg.title()?, None)
            };

            let session = WindowSession {
                title,
                url,
                window: fg,
            };
            return Ok(Some(session));
        }
        Ok(None)
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        let state = self.browser_state.blocking_read();
        if let Some(session) =
            Self::foreground_window_session(&self.browser, &state, self.track_incognito)?
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
