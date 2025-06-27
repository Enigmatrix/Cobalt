use std::marker::PhantomData;

use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, SetWindowsHookExW, UnhookWindowsHookEx, WINDOWS_HOOK_ID,
};

/// Instance of a Windows hook attached to a [WindowsHookType].
pub struct WindowsHook<T: WindowsHookType> {
    hook: HHOOK,
    _t: PhantomData<T>,
}

/// Trait for the type of Windows Hook.
pub trait WindowsHookType {
    /// Callback for the [WindowsHook].
    fn callback(code: i32, wparam: WPARAM, lparam: LPARAM);
    /// Id for the [WindowsHook], the type of Windows Hook being used.
    fn id() -> WINDOWS_HOOK_ID;
}

impl<T: WindowsHookType> WindowsHook<T> {
    /// Global hook for the given hook type.
    pub fn global() -> Result<WindowsHook<T>> {
        let hook = unsafe {
            SetWindowsHookExW(T::id(), Some(Self::trampoline), None, 0)
                .context("create global windows hook")?
        };
        Ok(WindowsHook {
            hook,
            _t: PhantomData,
        })
    }

    unsafe extern "system" fn trampoline(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        T::callback(ncode, wparam, lparam);
        unsafe { CallNextHookEx(None, ncode, wparam, lparam) }
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
