use std::ffi::c_void;

use util::config::Config;
use util::error::{bail, Context, Result};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WINDOWS_HOOK_ID, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
};

use crate::objects::{Duration, Timestamp, WindowsHook, WindowsHookType};

/// Watches for user interaction and notifies when the user becomes idle or active,
/// including counting mouse_clicks and key_presses.
pub struct InteractionWatcher {
    max_idle_duration: Duration,
    active: bool,

    // the below will be modified by Windows Hooks
    last_interaction: Timestamp,
    mouse_clicks: i64,
    key_strokes: i64,

    // Windows Hooks
    _mouse_hook: WindowsHook<MouseLL>,
    _keyboard_hook: WindowsHook<KeyboardLL>,
}

/// Event for change in interaction state.
pub enum InteractionChangedEvent {
    BecameIdle {
        at: Timestamp,
        recorded_mouse_clicks: i64,
        recorded_key_presses: i64,
    },
    BecameActive {
        at: Timestamp,
    },
}

static mut INTERACTION_INSTANCE: Option<InteractionWatcher> = None;

fn interaction_instance() -> &'static mut InteractionWatcher {
    unsafe { INTERACTION_INSTANCE.as_mut().unwrap() }
}

impl InteractionWatcher {
    /// Create a new [InteractionWatcher] with the specified [Config] and current [Timestamp].
    pub fn init(config: &Config, at: Timestamp) -> Result<&'static mut Self> {
        if unsafe { INTERACTION_INSTANCE.is_some() } {
            bail!("InteractionWatcher already initialized");
        }
        let instance = Self {
            max_idle_duration: config.max_idle_duration().into(),
            active: true,
            last_interaction: at,
            mouse_clicks: 0,
            key_strokes: 0,
            _mouse_hook: WindowsHook::global().context("mouse ll windows hook")?,
            _keyboard_hook: WindowsHook::global().context("keyboard ll windows hook")?,
        };
        unsafe { INTERACTION_INSTANCE = Some(instance) };
        Ok(interaction_instance())
    }

    /// Poll for a new [`InteractionChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<InteractionChangedEvent>> {
        let interaction_gap_duration = at - self.last_interaction;
        if self.active {
            if interaction_gap_duration > self.max_idle_duration {
                let recorded_mouse_clicks = self.mouse_clicks;
                let recorded_key_presses = self.key_strokes;
                self.mouse_clicks = 0;
                self.key_strokes = 0;
                self.active = false;
                Ok(Some(InteractionChangedEvent::BecameIdle {
                    at,
                    recorded_mouse_clicks,
                    recorded_key_presses,
                }))
            } else {
                Ok(None)
            }
        } else if interaction_gap_duration < self.max_idle_duration {
            self.active = true;
            Ok(Some(InteractionChangedEvent::BecameActive { at }))
        } else {
            Ok(None)
        }
    }

    /// Interact with this watcher
    pub fn interact(&mut self, timestamp: Timestamp) {
        self.last_interaction = self.last_interaction.max(timestamp)
    }
}

pub struct MouseLL;

impl WindowsHookType for MouseLL {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) {
        let lparam = unsafe { &mut *(lparam.0 as *mut c_void as *mut MSLLHOOKSTRUCT) };
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags & LLKHF_INJECTED.0 == 0 {
            interaction_instance().interact(Timestamp::from_event_millis(lparam.time));

            let mouse_ev = wparam.0 as u32;
            if mouse_ev == WM_LBUTTONDOWN
                || mouse_ev == WM_RBUTTONDOWN
                || mouse_ev == WM_MBUTTONDOWN
                || mouse_ev == WM_XBUTTONDOWN
            {
                interaction_instance().mouse_clicks += 1;
            }
        }
    }

    fn id() -> WINDOWS_HOOK_ID {
        WH_MOUSE_LL
    }
}

pub struct KeyboardLL;

impl WindowsHookType for KeyboardLL {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) {
        let lparam = unsafe { &mut *(lparam.0 as *mut c_void as *mut KBDLLHOOKSTRUCT) };
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags.0 & LLKHF_INJECTED.0 == 0 {
            interaction_instance().interact(Timestamp::from_event_millis(lparam.time));

            if wparam.0 as u32 == WM_KEYDOWN {
                interaction_instance().key_strokes += 1;
            }
        }
    }

    fn id() -> WINDOWS_HOOK_ID {
        WH_KEYBOARD_LL
    }
}
