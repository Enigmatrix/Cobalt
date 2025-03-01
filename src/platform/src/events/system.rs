use util::error::Result;
use util::tracing::ResultTraceExt;
use windows::Win32::UI::WindowsAndMessaging::{ENDSESSION_LOGOFF, WM_ENDSESSION};

use crate::objects::MessageWindow;

/// System events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Shutdown
    Shutdown,
    /// Logoff
    Logoff,
}

/// Watcher for system events
pub struct SystemEventWatcher {}

impl SystemEventWatcher {
    /// Create a new system watcher
    pub fn new(
        message_window: &MessageWindow,
        // Callback is used instead of a channel because we want it to be processed ASAP.
        mut callback: impl FnMut(SystemEvent) -> Result<()> + 'static,
    ) -> Result<Self> {
        // TODO add logon
        message_window.add_callback(Box::new(move |_, msg, wparam, lparam| {
            if msg == WM_ENDSESSION {
                if lparam.0 as u32 & ENDSESSION_LOGOFF != 0 {
                    callback(SystemEvent::Logoff).warn();
                } else {
                    // Cannot disambiguate between shutdown, restart etc.
                    callback(SystemEvent::Shutdown).warn();
                }
            }
            None
        }));
        Ok(Self {})
    }

    /// Poll for system watcher events
    pub fn poll(&mut self) -> Result<Option<SystemEvent>> {
        Ok(None)
    }
}
