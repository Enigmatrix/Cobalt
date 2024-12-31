use std::marker::PhantomData;

use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WINDOWS_HOOK_ID,
};

pub struct WindowsHook<T: WindowsHookType> {
    hook: HHOOK,
    _t: PhantomData<T>,
}

pub trait WindowsHookType {
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM);
    fn id() -> WINDOWS_HOOK_ID;
}

impl<T: WindowsHookType> WindowsHook<T> {
    pub fn global() -> Result<WindowsHook<T>> {
        let hook = unsafe {
            SetWindowsHookExW(T::id(), Some(Self::trampoline), HMODULE::default(), 0)
                .context("create global windows hook")?
        };
        Ok(WindowsHook {
            hook,
            _t: PhantomData,
        })
    }

    unsafe extern "system" fn trampoline(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        T::callback(ncode, wparam, lparam);
        CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
    }
}

impl<T: WindowsHookType> Drop for WindowsHook<T> {
    fn drop(&mut self) {
        unsafe {
            UnhookWindowsHookEx(self.hook)
                .context("unhook windows hook")
                .warn()
        };
    }
}
