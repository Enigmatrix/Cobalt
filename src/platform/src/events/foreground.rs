use util::config::Config;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;

use crate::objects::{BrowserDetector, Timestamp, Window};
/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
    browser: BrowserDetector,
    track_incognito: bool,
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
    pub fn new(session: WindowSession, config: &Config) -> Result<Self> {
        let browser = BrowserDetector::new()?;
        Ok(Self {
            session,
            browser,
            track_incognito: config.track_incognito(),
        })
    }

    /// Get the foreground window session.
    pub fn foreground_window_session(
        browser: &BrowserDetector,
        track_incognito: bool,
    ) -> Result<Option<WindowSession>> {
        if let Some(fg) = Window::foreground() {
            let title = fg.title()?;
            let (title, url) = if browser.is_chromium(&fg).warn() {
                let url = browser.chromium_url(&fg).context("get chromium url")?;
                if url.incognito && !track_incognito {
                    ("<Incognito>".to_string(), None)
                } else {
                    (title, url.url)
                }
            } else {
                (title, None)
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
        if let Some(session) = Self::foreground_window_session(&self.browser, self.track_incognito)?
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
