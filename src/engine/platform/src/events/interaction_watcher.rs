use utils::channels::Sender;
use utils::errors::*;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::WindowsAndMessaging::{
    HC_ACTION, HOOKPROC, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
};

use super::{Event, WindowsHook};
use crate::objects::{Duration, Timestamp};

#[derive(Debug)]
pub enum InteractionStateChange {
    Active,
    Idle { mouseclicks: u32, keystrokes: u32 },
}

// the u32s don't need to be atomic since the HookProcs are likely not re-entrant
pub struct InteractionWatcher {
    _mouse: WindowsHook,
    _keyboard: WindowsHook,
    max_gap: Duration,
    keystrokes: u32,
    mouseclicks: u32,
    last_interaction: Timestamp,
    idle: bool,
}

impl InteractionWatcher {
    pub fn new(max_gap: Duration, start: Timestamp, _mouse: HOOKPROC, _keyboard: HOOKPROC) -> Result<Self> {
        let _mouse =
            WindowsHook::global(WH_MOUSE_LL, _mouse).context("setup low-level mouse hook")?;
        let _keyboard = WindowsHook::global(WH_KEYBOARD_LL, _keyboard)
            .context("setup low-level keyboard hook")?;
        Ok(InteractionWatcher {
            _mouse,
            _keyboard,
            max_gap,
            keystrokes: 0,
            mouseclicks: 0,
            last_interaction: start,
            idle: true,
        })
    }

    pub fn reset_interactions(&mut self) {
        self.mouseclicks = 0;
        self.keystrokes = 0;
    }

    pub fn recent_interaction(&self, now: Timestamp) -> bool {
        now - self.last_interaction <= self.max_gap
    }

    pub fn trigger(&mut self, sender: &mut Sender<Event>, now: Timestamp) -> Result<()> {
        if self.idle && self.recent_interaction(now) {
            self.idle = false;
            sender
                .send(Event::InteractionStateChange {
                    at: now,
                    change: InteractionStateChange::Active,
                })
                .context("send interaction state change event (active)")?;
        } else if !self.idle && !self.recent_interaction(now) {
            self.idle = true;
            sender
                .send(Event::InteractionStateChange {
                    at: now,
                    change: InteractionStateChange::Idle {
                        mouseclicks: self.mouseclicks,
                        keystrokes: self.keystrokes,
                    },
                })
                .context("send interaction state change event (idle)")?;
            self.reset_interactions();
        }
        Ok(())
    }

    #[inline]
    pub fn trigger_mouse(
        &mut self,
        code: i32,
        wparam: WPARAM,
        lparam: &MSLLHOOKSTRUCT,
    ) -> Result<()> {
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags & LLKHF_INJECTED.0 == 0 {
            self.last_interaction = self
                .last_interaction
                .max(Timestamp::from_event_millis(lparam.time));
            let mouse_ev = wparam.0 as u32;
            if mouse_ev == WM_LBUTTONDOWN
                || mouse_ev == WM_RBUTTONDOWN
                || mouse_ev == WM_MBUTTONDOWN
                || mouse_ev == WM_XBUTTONDOWN
            {
                self.mouseclicks += 1;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn trigger_keyboard(
        &mut self,
        code: i32,
        wparam: WPARAM,
        lparam: &KBDLLHOOKSTRUCT,
    ) -> Result<()> {
        // we do not count fake, 'injected' keyboard events to be user keyboard clicks
        if code as u32 == HC_ACTION && lparam.flags.0 & LLKHF_INJECTED.0 == 0 {
            self.last_interaction = self
                .last_interaction
                .max(Timestamp::from_event_millis(lparam.time));
            if wparam.0 as u32 == WM_KEYDOWN {
                self.keystrokes += 1;
            }
        }
        Ok(())
    }
}
