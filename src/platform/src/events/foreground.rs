use util::error::Result;

use crate::objects::{Timestamp, Window};
/// Watches for foreground window changes, including session (title) changes
pub struct ForegroundEventWatcher {
    session: WindowSession,
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
    /// Foreground [Window] url, if browser
    pub url: Option<String>,
}

impl ForegroundEventWatcher {
    /// Create a new [WindowSession] with the starting [WindowSession].
    pub fn new(session: WindowSession) -> Result<Self> {
        Ok(Self { session })
    }

    /// Get the foreground window session.
    pub fn foreground_window_session() -> Result<Option<WindowSession>> {
        if let Some(fg) = Window::foreground() {
            let title = fg.title()?;

            // we don't know the url yet, so we set it to None
            let session = WindowSession {
                title,
                window: fg,
                url: None,
            };
            return Ok(Some(session));
        }
        Ok(None)
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        if let Some(session) = Self::foreground_window_session()? {
            if session == self.session {
                return Ok(None);
            }
            self.session = session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }
}
