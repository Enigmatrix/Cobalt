use engine_windows_bindings::windows::win32::{system_services::PWSTR, windows_and_messaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, HWND}};
use std::{fmt, hash};
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
            /* .field("title", &self.title())*/
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
    pub fn new(hwnd: HWND) -> Window {
        Window { hwnd }
    }

    pub fn title(&self) -> Result<&dyn Buffer, Win32Err> {
        Win32Err::clear_last_err(); // yes, actually important!
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        // fails if len == 0 && !Win32Err::last_err().is_success()
        if len == 0 {
            let err = Win32Err::last_err();
            if err.is_success() {
                Ok(&local::<0>())
            } else {
                Err(err)
            }
        } else {
            let mut buf = alloc(len as usize + 1);
            
            let written =
                win32!(zero = GetWindowTextW(self.hwnd, PWSTR(buf.as_mut_ptr()), len + 1))?;
            Ok(&buf.with_length(written as usize))
        }
    }

    pub fn foreground() -> Result<Window, Win32Err> {
        let fore = win32!(inner zero = GetForegroundWindow())?;
        Ok(Window::new(fore))
    }
}