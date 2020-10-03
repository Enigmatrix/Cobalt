macro_rules! expect {
    (true: $e: expr) => {{
        let val = unsafe { $e };
        if val == 0 {
            Err($crate::errors::last_win32_error())
        } else {
            Ok(val)
        }
    }};
    (non_null: $e: expr) => {{
        let val = unsafe { $e };
        if val.is_null() {
            Err($crate::errors::last_win32_error())
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
            Err(Error::from_kind($crate::errors::ErrorKind::HResult(val)))
        } else {
            Ok(val)
        }
    }};
}

macro_rules! ntstatus {
    ($e: expr) => {{
        let val = unsafe { $e };
        if val < 0 {
            Err(Error::from_kind($crate::errors::ErrorKind::NtStatus(val)))
        } else {
            Ok(val)
        }
    }};
}
