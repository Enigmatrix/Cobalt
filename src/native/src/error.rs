use crate::raw::errhandlingapi::{GetLastError, SetLastError};
use std::error::Error;
use std::fmt;

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

impl Win32Err {
    pub fn from(err: i32) -> Win32Err {
        Win32Err(err)
    }

    pub fn last_err() -> Win32Err {
        let err = Win32Err(unsafe { GetLastError() } as i32);
        unsafe { SetLastError(0) };
        err
    }

    pub fn is_success(&self) -> bool {
        self.0 == 0
    }
}

pub struct HResult(i32);

impl fmt::Display for HResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = std::io::Error::from_raw_os_error(self.0).to_string();
        write!(f, "HResult(0x{:x}): {}", self.0, err)
    }
}

impl fmt::Debug for HResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for HResult {}

impl HResult {
    pub fn new(code: i32) -> HResult {
        HResult(code)
    }
}

pub struct NtStatus(i32);

impl NtStatus {
    pub fn new(code: i32) -> NtStatus {
        NtStatus(code)
    }
}

impl fmt::Display for NtStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = std::io::Error::from_raw_os_error(self.0).to_string();
        write!(f, "NtStatus(0x{:x}): {}", self.0, err)
    }
}

impl fmt::Debug for NtStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for NtStatus {}

#[derive(Debug)]
pub struct WinRt(winrt::Error);
unsafe impl Send for WinRt {}
unsafe impl Sync for WinRt {}

impl fmt::Display for WinRt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<winrt::Error> for WinRt {
    fn from(x: winrt::Error) -> Self {
        Self(x)
    }
}

impl Error for WinRt {}

macro_rules! win32 {
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

macro_rules! hresult {
    ($e: expr) => {{
        let val = unsafe { $e } as i32;
        if val < 0 {
            Err($crate::error::HResult::new(val))
        } else if val != 0 {
            tracing::warn!("HRESULT WARN 0x{:0x}", val);
            Ok(())
        } else {
            Ok(())
        }
    }};
}

macro_rules! ntstatus {
    ($e: expr) => {{
        let val = unsafe { $e } as i32;
        if val != 0 {
            Err($crate::error::NtStatus::new(val))
        } else {
            Ok(())
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win32_err_6_gives_correct_output() {
        let err = Win32Err(6);
        assert_eq!(
            err.to_string(),
            "Win32(0x6): The handle is invalid. (os error 6)"
        )
    }
}
