use std::mem::MaybeUninit;
use utils::channels::Sender;
use utils::errors::*;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

use super::{ForegroundWatcher, WinEventArgs};

#[derive(Debug)]
pub enum Event {
    ForegroundSwitch { hwnd: HWND, timestamp: u32 },
}

pub struct TotalWatcher {
    foreground: ForegroundWatcher,
    pub(crate) sender: Sender<Event>,
}

static mut INSTANCE: MaybeUninit<TotalWatcher> = MaybeUninit::uninit();

impl TotalWatcher {
    pub fn new(sender: Sender<Event>) -> Result<&'static TotalWatcher> {
        let foreground = ForegroundWatcher::new(Some(TotalWatcher::foreground_watcher_callback))
            .context("setup foreground watcher")?;
        unsafe { INSTANCE = MaybeUninit::new(TotalWatcher { foreground, sender }) };
        Ok(unsafe { TotalWatcher::instance() })
    }

    unsafe fn instance() -> &'static Self {
        INSTANCE.assume_init_ref()
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
        watcher.foreground
            .trigger(
                watcher,
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
