#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Error {
    Win32(i32),
    HResult(i32),
    NtStatus(i32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Win32(r) => {
                let err = std::io::Error::from_raw_os_error(*r);
                let desc = err.to_string();
                write!(fmt, "Win32 ({}): {}", r, desc)
            }
            Self::HResult(r) => write!(fmt, "HResult ({})", r),
            Self::NtStatus(r) => write!(fmt, "NtStatus ({})", r),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(self, fmt)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn last_win32() -> Self {
        Error::Win32(unsafe { crate::os::api::errhandlingapi::GetLastError() as i32 })
    }

    pub fn successful(&self) -> bool {
        match self {
            &Self::Win32(r) => r == 0,
            Self::HResult(_) => todo!(),
            Self::NtStatus(_) => todo!(),
        }
    }
}

macro_rules! expect {
    (true: $e: expr) => {{
        let val = unsafe { $e };
        if val == 0 {
            Err($crate::os::error::Error::last_win32())
        } else {
            Ok(val)
        }
    }};
    (non_null: $e: expr) => {{
        let val = unsafe { $e };
        if val.is_null() {
            Err($crate::os::error::Error::last_win32())
        } else {
            Ok(val)
        }
    }};
}

// TODO 0 is not the only successful error code! refer to the SUCCESS macro
macro_rules! hresult {
    ($e: expr) => {{
        let val = unsafe { $e };
        if val < 0 {
            Err($crate::os::error::Error::HResult(val))
        } else {
            Ok(val)
        }
    }};
}

macro_rules! ntstatus {
    ($e: expr) => {{
        let val = unsafe { $e };
        if val < 0 {
            Err($crate::os::error::Error::NtStatus(val))
        } else {
            Ok(val)
        }
    }};
}
