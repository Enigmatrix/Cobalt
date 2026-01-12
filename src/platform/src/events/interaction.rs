use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};

use util::config::Config;
use util::error::{Context, Result, bail};
use util::time::ToTicks;
use util::tracing::debug;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WINDOWS_HOOK_ID, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
};

use crate::objects::{Duration, Timestamp, WindowsHook, WindowsHookType};

static INTERACTION_LAST_INTERACTION: AtomicI64 = AtomicI64::new(0);
static INTERACTION_MOUSE_CLICKS: AtomicI64 = AtomicI64::new(0);
static INTERACTION_KEY_STROKES: AtomicI64 = AtomicI64::new(0);

thread_local! {
    static INTERACTION_INSTANCE: Rc<RefCell<Option<InteractionWatcher>>> = Rc::new(RefCell::new(None));
}

/// Watches for user interaction and notifies when the user becomes idle or active,
/// including counting mouse_clicks and key_presses.
pub struct InteractionWatcher {
    max_idle_duration: Duration,
    active_start: Option<Timestamp>,
}

// THIS IS WRONG!
unsafe impl Send for InteractionWatcher {}
unsafe impl Sync for InteractionWatcher {}

/// Active Period Event
pub struct InteractionChangedEvent {
    /// Start of active period
    pub start: Timestamp,
    /// End of active period
    pub end: Timestamp,
    /// Mouse clicks in the active period
    pub mouse_clicks: i64,
    /// Recorded key presses in the active period
    pub key_strokes: i64,
}

impl InteractionWatcher {
    /// Create a new [InteractionWatcher] with the specified [Config] and current [Timestamp].
    /// Requires that [InteractionWatcherHooks] is running at some point in time, else the values will be wrong.
    pub fn init(config: &Config, at: Timestamp) -> Result<Rc<RefCell<Option<Self>>>> {
        // Check if already initialized
        if INTERACTION_INSTANCE.with(|instance| instance.borrow().is_some()) {
            bail!("InteractionWatcher already initialized");
        }

        // Create the new instance
        let instance = Self {
            max_idle_duration: config.max_idle_duration().into(),
            active_start: Some(at),
        };

        // Initialize the global instance if not already done
        let instance = INTERACTION_INSTANCE.with(|ginstance| {
            INTERACTION_LAST_INTERACTION.store(at.to_ticks(), Ordering::Relaxed);
            *ginstance.borrow_mut() = Some(instance);
            ginstance.clone()
        });
        Ok(instance)
    }

    /// Short-circuit the interaction watcher if the user is active.
    pub fn short_circuit(
        &mut self,
        is_active: bool,
        now: Timestamp,
    ) -> Option<InteractionChangedEvent> {
        if is_active {
            self.active_start = Some(now);
            None
        } else if let Some(start) = self.active_start {
            self.active_start = None;
            let recorded_mouse_clicks = INTERACTION_MOUSE_CLICKS.swap(0, Ordering::Relaxed);
            let recorded_key_presses = INTERACTION_KEY_STROKES.swap(0, Ordering::Relaxed);
            Some(InteractionChangedEvent {
                start,
                end: now,
                mouse_clicks: recorded_mouse_clicks,
                key_strokes: recorded_key_presses,
            })
        } else {
            // already not active
            None
        }
    }

    /// Poll for a new [`InteractionChangedEvent`].
    pub fn poll(&mut self, at: Timestamp) -> Result<Option<InteractionChangedEvent>> {
        let last_interaction =
            Timestamp::from_ticks(INTERACTION_LAST_INTERACTION.load(Ordering::Relaxed));
        let interaction_gap_duration = at - last_interaction;
        if let Some(start) = self.active_start {
            if interaction_gap_duration > self.max_idle_duration {
                debug!("became idle at {:?}", at);
                let recorded_mouse_clicks = INTERACTION_MOUSE_CLICKS.swap(0, Ordering::Relaxed);
                let recorded_key_presses = INTERACTION_KEY_STROKES.swap(0, Ordering::Relaxed);
                self.active_start = None;
                return Ok(Some(InteractionChangedEvent {
                    start,
                    end: last_interaction,
                    mouse_clicks: recorded_mouse_clicks,
                    key_strokes: recorded_key_presses,
                }));
            }
            return Ok(None);
        } else if interaction_gap_duration < self.max_idle_duration {
            debug!("became active at {:?}", at);
            self.active_start = Some(at);
        }
        Ok(None)
    }
}

/// Windows Hooks for the [InteractionWatcher]
pub struct InteractionWatcherHooks {
    _mouse_hook: WindowsHook<MouseLL>,
    _keyboard_hook: WindowsHook<KeyboardLL>,
}

impl InteractionWatcherHooks {
    /// Create a new [InteractionWatcherHooks]
    pub fn new() -> Result<Self> {
        Ok(Self {
            _mouse_hook: WindowsHook::global().context("mouse ll windows hook")?,
            _keyboard_hook: WindowsHook::global().context("keyboard ll windows hook")?,
        })
    }
}

/// Windows Hook for Low Level Mouse
pub struct MouseLL;

impl WindowsHookType for MouseLL {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) {
        let lparam = unsafe { &mut *(lparam.0 as *mut c_void as *mut MSLLHOOKSTRUCT) };
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags & LLKHF_INJECTED.0 == 0 {
            let ts = Timestamp::from_event_millis(lparam.time);
            INTERACTION_LAST_INTERACTION.fetch_max(ts.to_ticks(), Ordering::Relaxed);

            let mouse_ev = wparam.0 as u32;
            if mouse_ev == WM_LBUTTONDOWN
                || mouse_ev == WM_RBUTTONDOWN
                || mouse_ev == WM_MBUTTONDOWN
                || mouse_ev == WM_XBUTTONDOWN
            {
                INTERACTION_MOUSE_CLICKS.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn id() -> WINDOWS_HOOK_ID {
        WH_MOUSE_LL
    }
}

/// Windows Hook for Low Level Keyboard
pub struct KeyboardLL;

impl WindowsHookType for KeyboardLL {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) {
        let lparam = unsafe { &mut *(lparam.0 as *mut c_void as *mut KBDLLHOOKSTRUCT) };
        // we do not count fake, 'injected' mouse events to be user mouse clicks
        if code as u32 == HC_ACTION && lparam.flags.0 & LLKHF_INJECTED.0 == 0 {
            let ts = Timestamp::from_event_millis(lparam.time);
            INTERACTION_LAST_INTERACTION.fetch_max(ts.to_ticks(), Ordering::Relaxed);

            if wparam.0 as u32 == WM_KEYDOWN {
                INTERACTION_KEY_STROKES.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn id() -> WINDOWS_HOOK_ID {
        WH_KEYBOARD_LL
    }
}
