use utils::errors::*;
use windows::Win32::{
    Foundation::HINSTANCE,
    UI::WindowsAndMessaging::{
        SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, HOOKPROC, WINDOWS_HOOK_ID,
    },
};

pub struct WindowsHook {
    _hook: HHOOK,
}

impl WindowsHook {
    pub fn global(id: WINDOWS_HOOK_ID, proc: HOOKPROC) -> Result<Self> {
        let _hook = unsafe {
            SetWindowsHookExW(id, proc, HINSTANCE::default(), 0).context("setup windows hook")?
        };
        Ok(Self { _hook })
    }
}

impl Drop for WindowsHook {
    fn drop(&mut self) {
        // TODO handle error
        let _ = unsafe { UnhookWindowsHookEx(self._hook) };
    }
}
