use std::fmt;

use windows::Win32::Foundation::{
    ERROR_INSUFFICIENT_BUFFER, GetLastError, NO_ERROR, NTSTATUS, STATUS_INFO_LENGTH_MISMATCH,
    STATUS_SUCCESS, SetLastError, WIN32_ERROR,
};

/// Trait for converting a type to a [`Result`]
pub trait IntoResult {
    type Err: Sized;
    fn into_result(self) -> Result<(), Self::Err>;
}

/// Win32 Error (from GetLastError)
pub struct Win32Error {
    pub(crate) inner: WIN32_ERROR,
}

impl Win32Error {
    /// Get the last error raised by an Win32 API call (per-thread)
    fn get_last_error() -> WIN32_ERROR {
        unsafe { GetLastError() }
    }

    /// Get the last error raised by an Win32 API call as [`Win32Error`].
    pub fn last_error() -> Self {
        Self {
            inner: Self::get_last_error(),
        }
    }

    /// Get the last error raised by an Win32 API call as a [`Result`].
    /// If the error is [`NO_ERROR`], then it is [`Ok`]
    pub fn last_result() -> Result<(), Self> {
        let err = Self::get_last_error();
        if err == NO_ERROR {
            Ok(())
        } else {
            Err(Win32Error { inner: err })
        }
    }

    /// Check if this error is due to insufficient buffer size
    pub fn insufficient_size(&self) -> bool {
        self.inner == ERROR_INSUFFICIENT_BUFFER
    }

    /// Clear the last error (per-thread)
    pub fn clear_last_err() {
        unsafe { SetLastError(NO_ERROR) }
    }

    /// Convert into result
    pub fn into_result(self) -> Result<(), Self> {
        if self.inner == NO_ERROR {
            Ok(())
        } else {
            Err(self)
        }
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

impl From<WIN32_ERROR> for Win32Error {
    fn from(code: WIN32_ERROR) -> Self {
        Self { inner: code }
    }
}

impl IntoResult for WIN32_ERROR {
    type Err = Win32Error;
    fn into_result(self) -> Result<(), Self::Err> {
        Win32Error::from(self).into_result()
    }
}

/// [`NTSTATUS`] Error (from return code)
#[derive(Clone)]
pub struct NtError {
    inner: NTSTATUS,
}

impl NtError {
    /// Create a new [`NtError`] error from a [`NTSTATUS`] return code
    pub fn from(inner: NTSTATUS) -> Self {
        Self { inner }
    }

    /// Check if this error is due to insufficient buffer size
    pub fn insufficient_size(&self) -> bool {
        self.inner == STATUS_INFO_LENGTH_MISMATCH
    }

    /// Convert this error to a [`Result`].
    /// If the error is [`STATUS_SUCCESS`], then it is [`Ok`]
    pub fn into_result(self) -> Result<(), Self> {
        if self.inner == STATUS_SUCCESS {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl IntoResult for NTSTATUS {
    type Err = NtError;
    fn into_result(self) -> Result<(), Self::Err> {
        NtError::from(self).into_result()
    }
}

impl From<NtError> for Result<(), NtError> {
    fn from(value: NtError) -> Self {
        value.into_result()
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

/// Macro to wrap around Win32 API calls and check the last result.
/// Works on calls that return 0 on error using `win32!(val: call())`,
/// BOOL/wrapper types that are internally zero-value on error using `win32!(wrap: call())`,
/// null pointer return on error using `win32!(ptr: call())`,
#[macro_export]
macro_rules! win32 {
    (val: $e: expr) => {{
        let val = $e;
        if val == 0 {
            Err($crate::error::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
    (wrap: $e: expr) => {{
        let val = $e;
        if val.0 == 0 {
            Err($crate::error::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
    (ptr: $e: expr) => {{
        let val = $e;
        if val.is_null() {
            Err($crate::error::Win32Error::last_error())
        } else {
            Ok(val)
        }
    }};
}

/// Macro to repeatedly call a function with increasing buffer sizes until it succeeds,
/// until a limit.
#[macro_export]
macro_rules! adapt_size {
    ($ty:ty, $sz1i:ident: $sz1v:expr => $sz1e:expr, $buf1i:ident, $e: expr) => {{
        use $crate::buf::Buffer;

        let mut $sz1i = $sz1v;
        let mut $buf1i = $crate::buf::buf::<$ty>($sz1i as usize);
        let res: Result<_, _> = $e;
        if let Err(err) = &res {
            if err.insufficient_size() {
                if $sz1i > $sz1e {
                    util::error::bail!("exceeded max size");
                }
                let mut $buf1i = $crate::buf::buf::<$ty>($sz1i as usize);
                $e
            } else { res }
        } else {
            res
        }
    }};
}

/// Macro to repeatedly call a function with two buffers, increasing their sizes until it succeeds.
/// This is useful for Windows API functions that return data in two separate buffers.
#[macro_export]
macro_rules! adapt_size2 {
    ($ty:ty, $sz1i:ident: $sz1v:expr => $sz1e:expr, $buf1i:ident, $sz2i:ident: $sz2v:expr => $sz2e:expr, $buf2i:ident, $e: expr) => {{
        use $crate::buf::Buffer;

        let mut $sz1i = $sz1v;
        let mut $sz2i = $sz2v;
        let mut $buf1i = $crate::buf::buf::<$ty>($sz1i as usize);
        let mut $buf2i = $crate::buf::buf::<$ty>($sz2i as usize);
        let res: Result<_, _> = $e;
        if let Err(err) = &res {
            if err.insufficient_size() {
                if $sz1i > $sz1e || $sz2i > $sz2e {
                    util::error::bail!("exceeded max size");
                }
                let mut $buf1i = $crate::buf::buf::<$ty>($sz1i as usize);
                let mut $buf2i = $crate::buf::buf::<$ty>($sz2i as usize);
                $e
            } else { res }
        } else {
            res
        }
    }};
}
