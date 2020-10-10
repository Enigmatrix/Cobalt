macro_rules! win32 {
    (non_zero: $e: block) => {{
        let val = unsafe { $e };
        if val == 0 {
            Err(anyhow::Error::new($crate::errors::last_win32_error()))
        } else {
            Ok(val)
        }
    }};
    (non_null: $e: block) => {{
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
    ($e: block) => {{
        let val = unsafe { $e };
        if val < 0 {
            Err(anyhow::Error::new($crate::errors::AppError::HResult(val)))
        } else {
            Ok(val)
        }
    }};
}

macro_rules! ntstatus {
    ($e: block) => {{
        let val = unsafe { $e };
        if val < 0 {
            Err(anyhow::Error::new($crate::errors::AppError::NtStatus(val)))
        } else {
            Ok(val)
        }
    }};
}
