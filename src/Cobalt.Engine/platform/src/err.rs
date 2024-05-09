use std::fmt;
use windows::Win32::Foundation::{GetLastError, SetLastError, NO_ERROR, WIN32_ERROR};

pub struct Win32Error {
    inner: WIN32_ERROR,
}

impl Win32Error {
    fn get_last_error() -> WIN32_ERROR {
        unsafe { GetLastError() }
    }

    pub fn last_error() -> Self {
        Self {
            inner: Self::get_last_error(),
        }
    }

    pub fn last_result() -> Result<(), Self> {
        let err = Self::get_last_error();
        if err == NO_ERROR {
            Ok(())
        } else {
            Err(Win32Error { inner: err })
        }
    }

    pub fn clear_last_err() {
        unsafe { SetLastError(NO_ERROR) }
    }
}

impl fmt::Display for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = std::io::Error::from_raw_os_error(self.inner.0 as i32);
        write!(f, "Win32(0x{:x}): {}", self.inner.0, err)
    }
}
impl fmt::Debug for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Win32Error {}

#[macro_export]
macro_rules! win32 {
    (val: $e: expr) => {{
        let val = $e;
        if val == 0 {
            Err($crate::err::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
    (wrap: $e: expr) => {{
        let val = $e;
        if val.0 == 0 {
            Err($crate::err::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
    (ptr: $e: expr) => {{
        let val = $e;
        if val.is_null() {
            Err($crate::err::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
}
