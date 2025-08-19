use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows_core::{HSTRING, PCWSTR};

use crate::error::Win32Error;

/// Single instance state held by application
pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    /// Create a new single instance state
    pub fn new(name: &str) -> Result<Option<Self>> {
        let name_hstring = HSTRING::from(name);
        let name_pcwstr = PCWSTR::from_raw(name_hstring.as_ptr());
        let handle = unsafe { CreateMutexW(None, true, name_pcwstr)? };
        let last_err = Win32Error::last_error();
        Win32Error::clear_last_err();
        if last_err.inner == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle).context("close existing single instance app state")? };
            Ok(None)
        } else {
            Ok(Some(SingleInstance { handle }))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle)
                .context("close single instance app state")
                .error()
        };
    }
}
