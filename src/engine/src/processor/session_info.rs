use anyhow::*;
use native::watchers::*;
use native::wrappers::*;

#[derive(Debug)]
pub struct SessionInfo {
    closed_watcher: window_closed::Watcher,
}

impl SessionInfo {
    pub fn new(window: Window, closed_watcher: window_closed::Watcher) -> Result<SessionInfo> {
        Ok(SessionInfo { closed_watcher })
    }
}
