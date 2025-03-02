use util::error::{Context, Result};
use util::tracing::{info, ResultTraceExt};
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    ENDSESSION_LOGOFF, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, WM_ENDSESSION, WM_POWERBROADCAST,
    WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

use crate::objects::MessageWindow;

/// System events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Shutdown
    Shutdown,
    /// Logoff
    Logoff,
    /// Lock
    Lock,
    /// Unlock
    Unlock,
    /// Suspend
    Suspend,
    /// Resume
    Resume,
    // TODO sleep, hibernate, monitor on / monitor off
}

impl SystemEvent {
    /// Check if the event makes the system inactive
    pub fn inactive(&self) -> bool {
        match self {
            SystemEvent::Shutdown => true,
            SystemEvent::Logoff => true,
            SystemEvent::Lock => true,
            SystemEvent::Suspend => true,
            _ => false,
        }
    }
}
/// Watcher for system events
pub struct SystemEventWatcher<'a> {
    message_window: &'a MessageWindow,
}

impl<'a> SystemEventWatcher<'a> {
    /// Create a new system watcher
    pub fn new(
        message_window: &'a MessageWindow,
        // Callback is used instead of a channel because we want it to be processed ASAP.
        mut callback: impl FnMut(SystemEvent) -> Result<()> + 'static,
    ) -> Result<Self> {
        // TODO add logon

        unsafe {
            WTSRegisterSessionNotification(message_window.handle(), NOTIFY_FOR_THIS_SESSION)
                .context("Failed to register session notification")?
        };

        message_window.add_callback(Box::new(move |_, msg, wparam, lparam| {
            if msg == WM_ENDSESSION {
                if lparam.0 as u32 & ENDSESSION_LOGOFF != 0 {
                    callback(SystemEvent::Logoff).warn();
                } else {
                    // Cannot disambiguate between shutdown, restart etc.
                    callback(SystemEvent::Shutdown).warn();
                }
            } else if msg == WM_WTSSESSION_CHANGE {
                if wparam.0 as u32 == WTS_SESSION_LOCK {
                    callback(SystemEvent::Lock).warn();
                } else if wparam.0 as u32 == WTS_SESSION_UNLOCK {
                    callback(SystemEvent::Unlock).warn();
                } else {
                    info!("SESSION: unknown event {:x}", wparam.0);
                }
            } else if msg == WM_POWERBROADCAST {
                if wparam.0 as u32 == PBT_APMSUSPEND {
                    callback(SystemEvent::Suspend).warn();
                } else if wparam.0 as u32 == PBT_APMRESUMESUSPEND {
                    callback(SystemEvent::Resume).warn();
                } else {
                    info!("POWER: unknown event {:x}", wparam.0);
                }
            }
            None
        }));
        Ok(Self { message_window })
    }

    /// Poll for system watcher events
    pub fn poll(&mut self) -> Result<Option<SystemEvent>> {
        Ok(None)
    }
}

impl<'a> Drop for SystemEventWatcher<'a> {
    fn drop(&mut self) {
        unsafe {
            WTSUnRegisterSessionNotification(self.message_window.handle())
                .context("Failed to unregister session notification")
                .error()
        };
    }
}
