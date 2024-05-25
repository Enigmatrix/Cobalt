use util::error::{Context, Result};

use crate::objects::{Timestamp, Window};

pub struct ForegroundEventWatcher {
    session: WindowSession,
}

pub struct ForegroundChangedEvent {
    pub at: Timestamp,
    pub session: WindowSession,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WindowSession {
    pub window: Window,
    pub title: String,
}

impl WindowSession {
    pub fn new(window: Window) -> Result<Self> {
        let title = window.title()?;
        Ok(Self { window, title })
    }
}

impl ForegroundEventWatcher {
    pub fn new(session: WindowSession) -> Result<Self> {
        Ok(Self { session })
    }

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
