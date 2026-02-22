use util::error::{Context, Result};

use crate::browser::{ArcBrowser, Browser};
use crate::objects::{Timestamp, Window};

/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
    browser: ArcBrowser,
    track_incognito: bool,
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
    /// Create a new [ForegroundEventWatcher] with the starting [WindowSession].
    pub fn new(session: WindowSession, track_incognito: bool, browser: ArcBrowser) -> Self {
        Self {
            session,
            browser,
            track_incognito,
        }
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        if let Some(session) =
            Self::foreground_window_session(&*self.browser, self.track_incognito)?
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
        browser: &dyn Browser,
        track_incognito: bool,
    ) -> Result<Option<ForegroundWindowSessionInfo>> {
        let Some(window) = Window::foreground() else {
            return Ok(None);
        };

        let result = browser
            .identify(&window)
            .context("browser identify window")?;

        let (title, url, fetched_path) = if let Some(r) = result {
            if r.info.is_incognito && !track_incognito {
                ("<Incognito>".to_string(), None, r.resolved_info.exe_path)
            } else {
                (window.title()?, Some(r.info.url), r.resolved_info.exe_path)
            }
        } else {
            (window.title()?, None, None)
        };

        let window_session = WindowSession { title, url, window };
        let session = ForegroundWindowSessionInfo {
            window_session,
            fetched_path,
        };
        Ok(Some(session))
    }
}
