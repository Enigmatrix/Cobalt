use util::error::{Context, Result};
use util::tracing::{info, ResultTraceExt};
use windows::Win32::System::Power::{
    RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY,
    POWERBROADCAST_SETTING,
};
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::System::SystemServices::GUID_MONITOR_POWER_ON;
use windows::Win32::UI::WindowsAndMessaging::{
    DEVICE_NOTIFY_WINDOW_HANDLE, ENDSESSION_LOGOFF, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
    PBT_POWERSETTINGCHANGE, WM_ENDSESSION, WM_POWERBROADCAST, WM_WTSSESSION_CHANGE,
    WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

use crate::objects::{MessageWindow, Timestamp};

/// System events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Shutdown
    Shutdown = 0,
    /// Logoff
    Logoff = 1,
    /// Lock (Win+L)
    Lock = 2,
    /// Unlock
    Unlock = 3,
    /// Suspend
    Suspend = 4,
    /// Resume
    Resume = 5,
    /// Monitor on
    MonitorOn = 6,
    /// Monitor off
    MonitorOff = 7,
    // TODO sleep, hibernate (or are these just suspend)

    // no need for logon event - duing logout programs are killed
}

impl From<&SystemEvent> for i64 {
    fn from(event: &SystemEvent) -> Self {
        event.clone() as i64
    }
}

/// System state
#[derive(Debug, Clone, Default)]
pub struct SystemState {
    /// Is shutdown
    is_shutdown: bool,
    /// Is logoff
    is_logoff: bool,
    /// Is lock
    is_lock: bool,
    /// Is suspend
    is_suspend: bool,
    /// Is monitor off
    is_monitor_off: bool,
}

impl SystemState {
    /// Check if the system is active
    pub fn is_active(&self) -> bool {
        !self.is_shutdown
            && !self.is_logoff
            && !self.is_lock
            && !self.is_suspend
            && !self.is_monitor_off
    }
}

/// System state event
#[derive(Debug, Clone)]
pub struct SystemStateEvent {
    /// Timestamp
    pub timestamp: Timestamp,
    /// System state
    pub state: SystemState,
    /// Event
    pub event: SystemEvent,
}

/// Watcher for system events
pub struct SystemEventWatcher<'a> {
    message_window: &'a MessageWindow,
    monitor_power_notification: HPOWERNOTIFY,
}

impl<'a> SystemEventWatcher<'a> {
    /// Create a new system watcher
    pub fn new(
        message_window: &'a MessageWindow,
        // Callback is used instead of a channel because we want it to be processed ASAP.
        mut callback: impl FnMut(SystemStateEvent) -> Result<()> + 'static,
    ) -> Result<Self> {
        let hwnd = message_window.handle();

        unsafe {
            WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION)
                .context("Failed to register session notification")?
        };

        // Register for power setting notifications
        // let _console_display = unsafe {
        //     RegisterPowerSettingNotification(
        //         hwnd,
        //         &GUID_CONSOLE_DISPLAY_STATE,
        //         DEVICE_NOTIFY_WINDOW_HANDLE,
        //     )
        // };

        // let _session_display = unsafe {
        //     RegisterPowerSettingNotification(
        //         hwnd,
        //         &GUID_SESSION_DISPLAY_STATUS,
        //         DEVICE_NOTIFY_WINDOW_HANDLE,
        //     )
        // };

        let monitor_power_notification = unsafe {
            RegisterPowerSettingNotification(
                hwnd,
                &GUID_MONITOR_POWER_ON,
                DEVICE_NOTIFY_WINDOW_HANDLE,
            )
            .context("Failed to register monitor power notification")?
        };

        // let _system_away = unsafe {
        //     RegisterPowerSettingNotification(
        //         hwnd,
        //         &GUID_LIDSWITCH_STATE_CHANGE,
        //         DEVICE_NOTIFY_WINDOW_HANDLE,
        //     )
        // };

        let mut state = SystemState::default();

        message_window.add_callback(Box::new(move |_, msg, wparam, lparam| {
            let timestamp = Timestamp::now();
            if msg == WM_ENDSESSION {
                if lparam.0 as u32 & ENDSESSION_LOGOFF != 0 {
                    state.is_logoff = true;
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Logoff,
                    })
                    .warn();
                } else {
                    state.is_shutdown = true;
                    // Cannot disambiguate between shutdown, restart etc.
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Shutdown,
                    })
                    .warn();
                }
            } else if msg == WM_WTSSESSION_CHANGE {
                if wparam.0 as u32 == WTS_SESSION_LOCK {
                    state.is_lock = true;
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Lock,
                    })
                    .warn();
                } else if wparam.0 as u32 == WTS_SESSION_UNLOCK {
                    state.is_lock = false;
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Unlock,
                    })
                    .warn();
                } else {
                    info!("SESSION: unknown event {:x}", wparam.0);
                }
            } else if msg == WM_POWERBROADCAST {
                if wparam.0 as u32 == PBT_APMSUSPEND {
                    state.is_suspend = true;
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Suspend,
                    })
                    .warn();
                } else if wparam.0 as u32 == PBT_APMRESUMESUSPEND {
                    state.is_suspend = false;
                    callback(SystemStateEvent {
                        timestamp,
                        state: state.clone(),
                        event: SystemEvent::Resume,
                    })
                    .warn();
                } else if wparam.0 as u32 == PBT_POWERSETTINGCHANGE {
                    // Extract power setting information
                    let setting_info =
                        unsafe { (lparam.0 as *const POWERBROADCAST_SETTING).as_ref() };
                    if let Some(setting_info) = setting_info {
                        let power_guid = setting_info.PowerSetting;

                        if power_guid == GUID_MONITOR_POWER_ON {
                            let monitor_on =
                                unsafe { *(&setting_info.Data[0] as *const _ as *const u32) } != 0;
                            state.is_monitor_off = !monitor_on;
                            if monitor_on {
                                callback(SystemStateEvent {
                                    timestamp,
                                    state: state.clone(),
                                    event: SystemEvent::MonitorOn,
                                })
                                .warn();
                            } else {
                                callback(SystemStateEvent {
                                    timestamp,
                                    state: state.clone(),
                                    event: SystemEvent::MonitorOff,
                                })
                                .warn();
                            }
                        }
                        // else if power_guid == GUID_CONSOLE_DISPLAY_STATE {
                        //     let display_state =
                        //         unsafe { *(&setting_info.Data[0] as *const _ as *const u32) };
                        //     match display_state {
                        //         0 => info!("CONSOLE DISPLAY: Display is off"),
                        //         1 => info!("CONSOLE DISPLAY: Display is on"),
                        //         2 => info!("CONSOLE DISPLAY: Display is dimmed"),
                        //         _ => {}
                        //     }
                        // } else if power_guid == GUID_SESSION_DISPLAY_STATUS {
                        //     let display_state =
                        //         unsafe { *(&setting_info.Data[0] as *const _ as *const u32) };
                        //     match display_state {
                        //         0 => info!("SESSION DISPLAY: Display is off"),
                        //         1 => info!("SESSION DISPLAY: Display is on"),
                        //         2 => info!("SESSION DISPLAY: Display is dimmed"),
                        //         _ => {}
                        //     }
                        // } else if power_guid == GUID_LIDSWITCH_STATE_CHANGE {
                        //     let lid_open =
                        //         unsafe { *(&setting_info.Data[0] as *const _ as *const u32) } != 0;
                        //     info!("LID: lid is {}", if lid_open { "OPEN" } else { "CLOSED" });
                        // }
                    } else {
                        info!("POWER: unknown event {:x}", wparam.0);
                    }
                }
            }
            None
        }));
        Ok(Self {
            message_window,
            monitor_power_notification,
        })
    }

    /// Poll for system watcher events
    pub fn poll(&mut self) -> Result<Option<SystemEvent>> {
        Ok(None)
    }
}

impl Drop for SystemEventWatcher<'_> {
    fn drop(&mut self) {
        unsafe {
            UnregisterPowerSettingNotification(self.monitor_power_notification)
                .context("Failed to unregister monitor power notification")
                .error();
        }
        unsafe {
            WTSUnRegisterSessionNotification(self.message_window.handle())
                .context("Failed to unregister session notification")
                .error()
        };
    }
}
