use util::error::{Context, Result};

use crate::objects::{Timestamp, Window};

pub struct ForegroundEventWatcher {
    session: WindowSession,
}

pub struct ForegroundChangedEvent {
    pub at: Timestamp,
    pub session: WindowSession,
}

/// A session of a [Window]. Each session has a unique title.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    pub window: Window,
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
        if let Some(fg) = Window::foreground() {
            let session = WindowSession::new(fg).context("foreground window session")?;
            if session == self.session {
                return Ok(None);
            }

            self.session = session.clone();
            return Ok(Some(ForegroundChangedEvent { at, session }));
        }
        Ok(None)
    }
}
