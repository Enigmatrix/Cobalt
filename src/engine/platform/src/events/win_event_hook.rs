use crate::*;
use utils::errors::*;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND},
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK, WINEVENTPROC},
        WindowsAndMessaging::WINEVENT_OUTOFCONTEXT,
    },
};

pub struct WinEventHook {
    hook: HWINEVENTHOOK,
}

pub struct WinEventArgs {
    pub hwineventhook: HWINEVENTHOOK,
    pub event: u32,
    pub hwnd: HWND,
    pub idobject: i32,
    pub idchild: i32,
    pub ideventthread: u32,
    pub dwmseventtime: u32,
}

impl WinEventHook {
    pub fn global(event: u32, proc: WINEVENTPROC) -> Result<Self> {
        let hook = win32!(non_zero: unsafe {
            SetWinEventHook(
                event,
                event,
                HINSTANCE::default(),
                proc,
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        })
        .context("setup native WinEventHook")?;
        Ok(Self { hook })
    }
}

impl Drop for WinEventHook {
    fn drop(&mut self) {
        win32!(non_zero: unsafe { UnhookWinEvent(self.hook) } ).expect("drop WinEventHook");
    }
}
