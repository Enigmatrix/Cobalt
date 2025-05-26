use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use crate::objects::{BrowserDetector, Timestamp, Window};
/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
    browser: BrowserDetector,
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

impl WindowSession {
    /// Create a new [WindowSession] from a [Window].
    pub fn new(window: Window, url: Option<String>) -> Result<Self> {
        let title = window.title()?;
        Ok(Self { window, title, url })
    }
}

impl ForegroundEventWatcher {
    /// Create a new [WindowSession] with the starting [WindowSession].
    pub fn new(session: WindowSession) -> Result<Self> {
        let browser = BrowserDetector::new()?;
        Ok(Self { session, browser })
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        if let Some(fg) = Window::foreground() {
            let url = if self.browser.is_chromium(&fg).warn() {
                self.browser.chromium_url(&fg).context("get chromium url")?
            } else {
                None
            };

            let session = WindowSession::new(fg, url).context("foreground window session")?;
            if session == self.session {
                return Ok(None);
            }

            self.session = session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }
}
