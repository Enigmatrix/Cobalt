use engine_windows_bindings::windows::win32::{system_services::PWSTR, windows_and_messaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, HWND}};
use std::{ffi::OsString, fmt, hash};
use crate::error::{Win32Err};
use crate::buffer::*;
use crate::win32;

#[derive(Clone)]
pub struct Window {
    hwnd: HWND
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("hwnd", &self.hwnd)
            .field("title", &self.title())
            /* .field("class", &self.class()) */
            .finish()
    }
}

impl PartialEq<Window> for Window {
    fn eq(&self, other: &Window) -> bool {
        self.hwnd == other.hwnd
    }
}

impl Eq for Window {}

impl PartialEq<HWND> for Window {
    fn eq(&self, other: &HWND) -> bool {
        self.hwnd == *other
    }
}

impl hash::Hash for Window {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: hash::Hasher,
    {
        hasher.write_usize(self.hwnd.0 as usize);
    }
}


impl Window {
    pub fn new(hwnd: HWND) -> Result<Window, Win32Err> {
        Ok(Window { hwnd })
    }

    pub fn title(&self) -> Result<OsString, Win32Err> {
        Win32Err::clear_last_err(); // yes, actually important!
        
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        if len == 0 {
            Win32Err::last_result().map(|_| OsString::new())
        } else {
            let mut buf = alloc(len as usize + 1);
            
            let written =
                win32!(non_zero: GetWindowTextW(self.hwnd, PWSTR(buf.as_mut_ptr()), len + 1))?;
            Ok(buf.with_length(written as usize).as_os_string())
        }
    }

    pub fn foreground() -> Result<Window, Win32Err> {
        Window::new(win32!(non_zero: inner GetForegroundWindow())?)
    }
}