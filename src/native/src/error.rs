use crate::raw::errhandlingapi::{GetLastError, SetLastError};
use std::error::Error;
use std::fmt;
use util::*;

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
        Win32Err::clear_last_err();
        err
    }

    pub fn clear_last_err() {
        unsafe { SetLastError(0) };
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

/// Wrap winrt::Error and make it Send and Sync (actually safe)
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

pub trait WinRtExt<T> {
    fn winrt_context<C>(self, context: C) -> Result<T, util::Error>
    where
        C: fmt::Display + Send + Sync + 'static;
    fn winrt_with_context<C, F>(self, f: F) -> Result<T, util::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
    fn into_std(self) -> std::io::Result<T>;
}

impl<T> WinRtExt<T> for Result<T, winrt::Error> {
    fn winrt_context<C>(self, context: C) -> Result<T, util::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(WinRt::from).context(context)
    }

    fn winrt_with_context<C, F>(self, context: F) -> Result<T, util::Error>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(WinRt::from).with_context(context)
    }

    fn into_std(self) -> std::io::Result<T> {
        self.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))
    }
}

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
            util::log::warn!("HRESULT WARN 0x{:0x}", val);
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
