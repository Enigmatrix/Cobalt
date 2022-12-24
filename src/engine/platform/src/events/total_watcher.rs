use std::ffi::c_void;
use std::mem::MaybeUninit;
use utils::channels::Sender;
use utils::errors::*;
use windows::Win32::Foundation::{BOOLEAN, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, TranslateMessage, HHOOK, KBDLLHOOKSTRUCT, MSG,
    MSLLHOOKSTRUCT,
};

use crate::objects::{Duration, Timestamp, Window};

use super::{ForegroundWatcher, InteractionStateChange, InteractionWatcher, WinEventArgs};

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
    pub(crate) sender: Sender<Event>,
}

static mut INSTANCE: MaybeUninit<TotalWatcher> = MaybeUninit::uninit();

impl TotalWatcher {
    // Do not call twice
    pub fn new(sender: Sender<Event>) -> Result<&'static Self> {
        let foreground = ForegroundWatcher::new(Some(Self::foreground_watcher_callback))
            .context("setup foreground watcher")?;
        let interaction = InteractionWatcher::new(
            Duration::from_millis(5_000), // make this configurable
            Some(Self::interaction_watcher_mouse_callback),
            Some(TotalWatcher::interaction_watcher_keyboard_callback),
            Some(TotalWatcher::interaction_watcher_timer_callback),
        )
        .context("setup interaction watcher")?;

        unsafe {
            INSTANCE = MaybeUninit::new(Self {
                foreground,
                interaction,
                sender,
            })
        };
        Ok(unsafe { Self::instance() })
    }

    unsafe fn instance() -> &'static mut Self {
        INSTANCE.assume_init_mut()
    }

    unsafe extern "system" fn interaction_watcher_timer_callback(_: *mut c_void, _: BOOLEAN) {
        let watcher = TotalWatcher::instance();
        watcher
            .interaction
            .trigger(&mut watcher.sender, Timestamp::now())
            .context("interaction watcher timer trigger")
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

    unsafe extern "system" fn foreground_watcher_callback(
        hwineventhook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        idobject: i32,
        idchild: i32,
        ideventthread: u32,
        dwmseventtime: u32,
    ) {
        let watcher = TotalWatcher::instance();
        watcher
            .foreground
            .trigger(
                &mut watcher.sender,
                WinEventArgs {
                    hwineventhook,
                    event,
                    hwnd,
                    idobject,
                    idchild,
                    ideventthread,
                    dwmseventtime,
                },
            )
            .context("trigger foreground watcher")
            .unwrap();
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
