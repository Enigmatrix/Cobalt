use std::ffi::c_void;
use std::mem::MaybeUninit;

use utils::channels::Sender;
use utils::errors::*;
use windows::Win32::Foundation::{BOOLEAN, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, TranslateMessage, HHOOK, KBDLLHOOKSTRUCT, MSG,
    MSLLHOOKSTRUCT,
};

use super::{ForegroundWatcher, InteractionStateChange, InteractionWatcher};
use crate::objects::{Duration, Timer, Timestamp, Window};

#[derive(Debug)]
pub enum Event {
    ForegroundSwitch {
        at: Timestamp,
        window: Window,
    },
    InteractionStateChange {
        at: Timestamp,
        change: InteractionStateChange,
    },
}

pub struct TotalWatcher {
    foreground: ForegroundWatcher,
    interaction: InteractionWatcher,
    _timer: Timer,
    pub(crate) sender: Sender<Event>,
}

static mut INSTANCE: MaybeUninit<TotalWatcher> = MaybeUninit::uninit();

impl TotalWatcher {
    // Do not call twice
    pub fn new(events_tx: Sender<Event>, start: Timestamp) -> Result<&'static Self> {
        // run until we get an actual foreground window
        let fg_window = loop {
            if let Some(f) = Window::foreground() {
                break f;
            };
        };

        let foreground = ForegroundWatcher::new(fg_window).context("setup foreground watcher")?;
        let interaction = InteractionWatcher::new(
            Duration::from_millis(5_000), // TODO make this configurable
            start,
            Some(Self::interaction_watcher_mouse_callback),
            Some(Self::interaction_watcher_keyboard_callback),
        )
        .context("setup interaction watcher")?;

        // TODO let this be configurable
        let freq = Duration::from_millis(200);

        let _timer = Timer::new(freq, freq, Some(TotalWatcher::timer_callback), None)
            .context("polling timer for interaction watcher")?;

        unsafe {
            INSTANCE = MaybeUninit::new(Self {
                foreground,
                interaction,
                _timer,
                sender: events_tx,
            })
        };
        Ok(unsafe { Self::instance() })
    }

    unsafe fn instance() -> &'static mut Self {
        INSTANCE.assume_init_mut()
    }

    unsafe extern "system" fn timer_callback(_: *mut c_void, _: BOOLEAN) {
        let watcher = TotalWatcher::instance();
        watcher
            .trigger()
            .context("total watcher timer trigger")
            .unwrap();
    }

    unsafe extern "system" fn interaction_watcher_mouse_callback(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        TotalWatcher::instance()
            .interaction
            .trigger_mouse(code, wparam, &*(lparam.0 as *const MSLLHOOKSTRUCT))
            .context("interaction watcher mouse trigger")
            .unwrap();
        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }

    unsafe extern "system" fn interaction_watcher_keyboard_callback(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        TotalWatcher::instance()
            .interaction
            .trigger_keyboard(code, wparam, &*(lparam.0 as *const KBDLLHOOKSTRUCT))
            .context("interaction watcher mouse trigger")
            .unwrap();
        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }

    pub fn trigger(&mut self) -> Result<()> {
        let now = Timestamp::now();
        self.interaction
            .trigger(&mut self.sender, now)
            .context("trigger interaction watcher")?;
        self.foreground
            .trigger(&mut self.sender, now)
            .context("trigger interaction watcher")?;
        Ok(())
    }

    pub fn run(&self) {
        let mut msg: MSG = Default::default();
        while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() } {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }
        // drop this value
        unsafe { INSTANCE = MaybeUninit::uninit() }
    }
}
