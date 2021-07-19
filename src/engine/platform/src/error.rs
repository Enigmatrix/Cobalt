use std::error::Error;
use std::fmt;

use bindings::Windows::Win32::{self, System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR}};

pub struct Win32Err(i32);

impl fmt::Display for Win32Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = std::io::Error::from_raw_os_error(self.0);
        write!(f, "Win32(0x{:x}): {}", self.0, err)
    }
}

impl fmt::Debug for Win32Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for Win32Err {}

impl From<i32> for Win32Err {
    fn from(v: i32) -> Self {
        Win32Err(v)
    }
}

impl From<u32> for Win32Err {
    fn from(v: u32) -> Self {
        Win32Err(v as i32)
    }
}

impl From<WIN32_ERROR> for Win32Err {
    fn from(v: WIN32_ERROR) -> Self {
        Win32Err(v.0 as i32)
    }
}

impl Win32Err {
    
    pub fn last_err() -> Win32Err {
        let err = Win32Err::from(unsafe { GetLastError() });
        Win32Err::clear_last_err();
        err
    }

    pub fn last_result() -> Result<(), Win32Err> {
        let err = Win32Err::last_err();
        if err.is_success() {
            Ok(())
        } else {
            Err(err)
        }
    }

    pub fn clear_last_err() {
        unsafe { SetLastError(0) };
    }

    pub fn is_success(&self) -> bool {
        self.0 == 0
    }
}

#[macro_export]
macro_rules! win32 {
    (non_zero: inner $e: expr) => {{
        let val = unsafe { $e };
        if val.0 == 0 {
            Err($crate::error::Win32Err::last_err())
        } else {
            Ok(val)
        }
    }};
    (non_zero: $e: expr) => {{
        let val = unsafe { $e };
        if val == 0 {
            Err($crate::error::Win32Err::last_err())
        } else {
            Ok(val)
        }
    }};
    (non_null: $e: expr) => {{
        let val = unsafe { $e };
        if val.is_null() {
            Err($crate::error::Win32Err::last_err())
        } else {
            Ok(val)
        }
    }};
}