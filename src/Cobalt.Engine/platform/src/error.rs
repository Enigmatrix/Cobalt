use std::fmt;
use windows::Win32::Foundation::{
    GetLastError, SetLastError, NO_ERROR, NTSTATUS, STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS,
    WIN32_ERROR,
};

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

#[derive(Clone)]
pub struct NtError {
    inner: NTSTATUS,
}

impl NtError {
    pub fn from(inner: NTSTATUS) -> Self {
        Self { inner }
    }

    pub fn insufficient_size(&self) -> bool {
        self.inner == STATUS_INFO_LENGTH_MISMATCH
    }

    pub fn to_result(&self) -> Result<(), Self> {
        if self.inner == STATUS_SUCCESS {
            Ok(())
        } else {
            Err(self.clone())
        }
    }
}

// We don't do any fancy strings for NTSTATUS as they are ill-defined e.g. 0xc0000005:
// ref: https://microsoft.public.win32.programmer.kernel.narkive.com/7QXg81R8/ntstatus-to-string
impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NtError(0x{:x})", self.inner.0)
    }
}
impl fmt::Debug for NtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for NtError {}

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

#[macro_export]
macro_rules! adapt_size {
    ($szi:ident, $bufi:ident -> $e: expr, $max_sz: expr) => {{
        use $crate::buf::Buffer;

        let mut $szi = 0;
        let $bufi = std::ptr::null_mut();
        let res: Result<_, _> = $e;
        if let Err(err) = &res {
            if err.insufficient_size() {
                if $szi > $max_sz {
                    util::error::bail!("exceeded max size");
                }
                let mut _buf = $crate::buf::buf::<u8>($szi as usize);
                let $bufi = _buf.as_mut_void();
                $e
            } else { res }
        } else {
            res
        }
    }};
}
