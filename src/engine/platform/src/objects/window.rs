use std::fmt;
use utils::errors::*;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, IsWindow,
    },
};

use crate::{
    buffers::{buf, Buffer},
    errors::Win32Error,
    win32,
};

#[derive(PartialEq, Eq, Clone)]
pub struct Window {
    hwnd: HWND,
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("hwnd", &format_args!("0x{:x}", self.hwnd.0))
            .field("title", &self.title())
            // .field("class", &self.class())
            // .field("aumid", &self.aumid())
            .finish()
    }
}

impl Window {
    pub fn new(hwnd: HWND) -> Self {
        Window { hwnd }
    }

    pub fn foreground() -> Option<Self> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0 == 0 {
            None
        } else {
            Some(Self::new(hwnd))
        }
    }

    /// Check whether this window still exists on the user's desktop
    pub fn exists(&self) -> bool {
        unsafe { IsWindow(self.hwnd).as_bool() }
    }

    /// Get the title of this window
    pub fn title(&self) -> Result<String> {
        Win32Error::clear_last_err(); // yes, actually important!
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        // fails if len == 0 && !Win32Err::last_err().is_success()
        if len == 0 {
            Win32Error::last_result()
                .map(|_| String::new())
                .context("bad window title length")
        } else {
            let mut buf = buf(len as usize + 1);
            let written =
                win32!(non_zero_num: unsafe { GetWindowTextW(self.hwnd, buf.as_bytes()) })
                    .context("cannot read window title")?;

            Ok(buf.with_length(written as usize).to_string_lossy())
        }
    }
}
