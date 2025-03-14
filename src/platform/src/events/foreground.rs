use util::error::{Context, Result};

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
}

impl WindowSession {
    /// Create a new [WindowSession] from a [Window].
    pub fn new(window: Window) -> Result<Self> {
        let title = window.title()?;
        Ok(Self { window, title })
    }
}

impl ForegroundEventWatcher {
    /// Create a new [WindowSession] with the starting [WindowSession].
    pub fn new(session: WindowSession) -> Result<Self> {
        Ok(Self { session })
    }

    /// Poll for a new [`ForegroundChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<ForegroundChangedEvent>> {
        let window = Window::foreground().unwrap_or_else(Window::desktop);
        let session = WindowSession::new(window).context("foreground window session")?;
        if session == self.session {
            Ok(None)
        } else {
            self.session = session.clone();
            Ok(Some(ForegroundChangedEvent { at, session }))
        }
    }
}
