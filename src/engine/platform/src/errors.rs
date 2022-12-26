use std::fmt;
use windows::Win32::Foundation::{
    GetLastError, SetLastError, ERROR_INSUFFICIENT_BUFFER, NO_ERROR, NTSTATUS,
    STATUS_INFO_LENGTH_MISMATCH, WIN32_ERROR,
};

pub struct Win32Error(WIN32_ERROR);

impl Win32Error {
    fn get_last_error() -> WIN32_ERROR {
        let err = unsafe { GetLastError() };
        // Self::clear_last_err();
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

    pub fn insufficient_size(&self) -> bool {
        let err = self.0;
        err == ERROR_INSUFFICIENT_BUFFER
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

pub struct NtError(NTSTATUS);

impl NtError {
    pub fn insufficient_size(&self) -> bool {
        let err = self.0;
        err == STATUS_INFO_LENGTH_MISMATCH
    }
}

// We don't do any fancy strings for NTSTATUS as they are ill-defined e.g. 0xc0000005:
// ref: https://microsoft.public.win32.programmer.kernel.narkive.com/7QXg81R8/ntstatus-to-string
impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NtError(0x{:x})", self.0 .0)
    }
}
impl fmt::Debug for NtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for NtError {}

impl From<windows::core::Error> for NtError {
    fn from(err: windows::core::Error) -> Self {
        // NTSTATUS is turned into a HRESULT in windows-rs by or-ing with 0x1000_0000.
        // e.g. STATUS_INFO_LENGTH_MISMATCH gets turned to 0xD0000004 instead of 0xC0000004.
        // TODO check when the windows-rs library doesn't do this strange transformation
        let status = err.code().0 & !0x1000_0000;
        NtError(NTSTATUS(status))
    }
}

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

#[macro_export]
macro_rules! repeat_size {
    ($szi: ident -> $e: expr, $start_sz: expr, $max_sz: expr) => {{
        let mut $szi = $start_sz;
        loop {
            let res: Result<_, _> = $e;
            if let Err(err) = &res && err.insufficient_size() {
                $szi *= 2;
                if $szi > $max_sz {
                    utils::errors::bail!("exceeded max size");
                }
            } else {
                break res;
            }
        }
    }};
}

#[macro_export]
macro_rules! repeat_twice {
    ($szi:ident, $bufi:ident -> $e: expr, $max_sz: expr) => {{
        use $crate::buffers::Buffer;

        let mut $szi = 0;
        let $bufi = std::ptr::null_mut();
        let res: Result<_, _> = $e;
        if let Err(err) = &res && err.insufficient_size() {
            if $szi > $max_sz {
                utils::errors::bail!("exceeded max size");
            }
            let mut _buf = $crate::buffers::buf::<u8>($szi as usize);
            let $bufi = _buf.as_mut_void();
            $e
        } else {
            res
        }
    }};
}
