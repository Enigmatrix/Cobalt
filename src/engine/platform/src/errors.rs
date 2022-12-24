use std::fmt;
use windows::Win32::Foundation::{GetLastError, SetLastError, NO_ERROR, WIN32_ERROR};

pub struct Win32Error(WIN32_ERROR);

impl Win32Error {
    fn get_last_error() -> WIN32_ERROR {
        let err = unsafe { GetLastError() };
        Self::clear_last_err();
        err
    }

    pub fn last_err() -> Self {
        Self(Self::get_last_error())
    }

    pub fn last_result() -> Result<(), Self> {
        let err = Self::get_last_error();
        if err == NO_ERROR {
            Ok(())
        } else {
            Err(Self(err))
        }
    }

    pub fn clear_last_err() {
        unsafe { SetLastError(NO_ERROR) }
    }
}

impl fmt::Display for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = std::io::Error::from_raw_os_error(self.0 .0 as i32);
        write!(f, "Win32(0x{:x}): {}", self.0 .0, err)
    }
}
impl fmt::Debug for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Win32Error {}

/// A macro to ease the use of checking for errors after a call to Win32 APIs.
/// Accepts return types such as integers (non_zero_num: <call>), integer-types that can't be converted back to an integer (non_zero: <call>)
/// and nullable pointer types (non_null: <call>)
#[macro_export]
macro_rules! win32 {
    (non_zero: $e: expr) => {{
        let val = $e;
        if val.0 == 0 {
            Err($crate::errors::Win32Error::last_err())
        } else {
            Ok(val)
        }
    }};
    (non_zero_num: $e: expr) => {{
        let val = $e;
        if val == 0 {
            Err($crate::errors::Win32Error::last_err())
        } else {
            Ok(val)
        }
    }};
    (non_null: $e: expr) => {{
        let val = $e;
        if val.is_null() {
            Err($crate::errors::Win32Error::last_err())
        } else {
            Ok(val)
        }
    }};
}
