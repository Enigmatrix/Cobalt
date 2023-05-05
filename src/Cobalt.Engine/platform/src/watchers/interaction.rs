use std::ffi::c_void;
use std::mem::MaybeUninit;

use common::errors::*;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WINDOWS_HOOK_ID, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
};

use crate::objects::{Duration, Timestamp, WindowsHook, WindowsHookType};

pub enum InteractionStateChange {
    Active,
    Idle { mouseclicks: u32, keystrokes: u32 },
}

// The reading/reset of mouseclicks+keystroke on the some other thread might race
// with the update on the callbacks by the Win32 EventLoop thread. But if they are the
// same thread, which is the case in engine, then there is no issue.

/// [Interaction] watcher (mouseclicks, keystrokes)
pub struct Interaction {
    mouseclicks: u32,
    keystrokes: u32,
    last_interaction: Timestamp,
    timeout: Duration,
    idle: bool,
    _mouse_hook: WindowsHook<MouseLL>,
    _keyboard_hook: WindowsHook<KeyboardLL>,
}

static mut INTERACTION_INSTANCE: MaybeUninit<Interaction> = MaybeUninit::uninit();

impl Interaction {
    /// Interact with this watcher
    pub fn interact(&mut self, timestamp: Timestamp) {
        self.last_interaction = self.last_interaction.max(timestamp)
    }

    /// Check if there is [InteractionStateChange] being triggered
    pub fn trigger(&mut self, now: Timestamp) -> Result<Option<InteractionStateChange>> {
        if self.idle && self.recent(now) {
            self.idle = false;
            return Ok(Some(InteractionStateChange::Active));
        } else if !self.idle && !self.recent(now) {
            self.idle = true;
            let change = InteractionStateChange::Idle {
                mouseclicks: self.mouseclicks,
                keystrokes: self.keystrokes,
            };
            self.reset();
            return Ok(Some(change));
        }
        Ok(None)
    }

    /// Check if our last interaction is recent according to the timeout
    fn recent(&self, now: Timestamp) -> bool {
        now - self.last_interaction <= self.timeout
    }

    fn reset(&mut self) {
        self.mouseclicks = 0;
        self.keystrokes = 0;
    }

    /// Initialize this global [Interaction] with new settings
    pub fn initialize<'a>(timeout: Duration, now: Timestamp) -> Result<&'a Self> {
        unsafe {
            INTERACTION_INSTANCE = MaybeUninit::new(Self {
                mouseclicks: 0,
                keystrokes: 0,
                last_interaction: now,
                timeout,
                idle: false,
                _mouse_hook: WindowsHook::global().context("mouse ll windows hook")?,
                _keyboard_hook: WindowsHook::global().context("keyboard ll windows hook")?,
            })
        };
        Ok(unsafe { INTERACTION_INSTANCE.assume_init_ref() })
    }
}

pub struct MouseLL;

impl WindowsHookType for MouseLL {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) {
        let lparam = unsafe { &mut *(lparam.0 as *mut c_void as *mut MSLLHOOKSTRUCT) };
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags & LLKHF_INJECTED.0 == 0 {
            unsafe {
                INTERACTION_INSTANCE
                    .assume_init_mut()
                    .interact(Timestamp::from_event_millis(lparam.time))
            };

            let mouse_ev = wparam.0 as u32;
            if mouse_ev == WM_LBUTTONDOWN
                || mouse_ev == WM_RBUTTONDOWN
                || mouse_ev == WM_MBUTTONDOWN
                || mouse_ev == WM_XBUTTONDOWN
            {
                unsafe { INTERACTION_INSTANCE.assume_init_mut().mouseclicks += 1 };
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
            unsafe {
                INTERACTION_INSTANCE
                    .assume_init_mut()
                    .interact(Timestamp::from_event_millis(lparam.time))
            };

            if wparam.0 as u32 == WM_KEYDOWN {
                unsafe { INTERACTION_INSTANCE.assume_init_mut().keystrokes += 1 };
            }
        }
    }

    fn id() -> WINDOWS_HOOK_ID {
        WH_KEYBOARD_LL
    }
}
