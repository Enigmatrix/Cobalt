use utils::errors::*;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, HOOKPROC, WINDOWS_HOOK_ID,
};

use crate::win32;

pub struct WindowsHook {
    _hook: HHOOK,
}

impl WindowsHook {
    pub fn global(id: WINDOWS_HOOK_ID, proc: HOOKPROC) -> Result<Self> {
        let _hook = unsafe {
            SetWindowsHookExW(id, proc, HINSTANCE::default(), 0).context("setup WindowsHook")
        }?;
        Ok(Self { _hook })
    }
}

impl Drop for WindowsHook {
    fn drop(&mut self) {
        win32!(non_zero: unsafe { UnhookWindowsHookEx(self._hook) }).expect("drop WindowsHook");
    }
}
