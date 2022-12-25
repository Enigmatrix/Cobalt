use std::fmt;
use utils::errors::*;
use windows::{
    core::{Vtable, GUID},
    Win32::{
        Foundation::HWND,
        Storage::EnhancedStorage::PKEY_AppUserModel_ID,
        System::Com::CoTaskMemFree,
        UI::{
            Shell::PropertiesSystem::{
                IPropertyStore, PropVariantToStringAlloc, SHGetPropertyStoreForWindow,
            },
            WindowsAndMessaging::{
                GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
                GetWindowThreadProcessId, IsWindow,
            },
        },
    },
};

use crate::{
    buffers::{buf, Buffer},
    errors::Win32Error,
    repeat_size, win32,
};

use super::PidTid;

#[derive(PartialEq, Eq, Clone)]
pub struct Window {
    hwnd: HWND,
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("hwnd", &format_args!("0x{:x}", self.hwnd.0))
            .field("title", &self.title())
            .field("class", &self.class())
            .field("aumid", &self.aumid())
            .finish()
    }
}

#[allow(non_upper_case_globals)]
const IID_IPropertyStore: GUID = GUID::from_values(
    0x886d8eeb,
    0x8cf2,
    0x4446,
    [0x8d, 0x02, 0xcd, 0xba, 0x1d, 0xbd, 0xcf, 0x99],
);

impl Window {
    pub fn new(hwnd: HWND) -> Self {
        Window { hwnd }
    }

    pub fn foreground() -> Option<Self> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0 == 0 {
            None
        } else {
            Some(Self::new(hwnd))
        }
    }

    /// Check whether this window still exists on the user's desktop
    pub fn exists(&self) -> bool {
        unsafe { IsWindow(self.hwnd).as_bool() }
    }

    /// Get the title of this window
    pub fn title(&self) -> Result<String> {
        Win32Error::clear_last_err(); // yes, actually important!
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        // fails if len == 0 && !Win32Err::last_err().is_success()
        if len == 0 {
            Win32Error::last_result()
                .map(|_| String::new())
                .context("bad window title length")
        } else {
            let mut buf = buf(len as usize + 1);
            let written =
                win32!(non_zero_num: unsafe { GetWindowTextW(self.hwnd, buf.as_bytes()) })
                    .context("cannot read window title")?;

            Ok(buf.with_length(written as usize).to_string_lossy())
        }
    }

    pub fn class(&self) -> Result<String> {
        repeat_size!(len -> {
            let mut buf = buf(len);
            win32!(non_zero_num: unsafe { GetClassNameW(self.hwnd, buf.as_bytes()) })
                .map(|new_len| buf.with_length(new_len as usize).to_string_lossy())
        }, 0x100, 0x100000)
        .context("get class name")
    }

    pub fn pid_tid(&self) -> Result<PidTid> {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) };
        let pidtid = PidTid { pid, tid };
        if pidtid.valid() {
            Ok(pidtid)
        } else {
            Err(Win32Error::last_err()).context("get window thread process id")
        }
    }

    pub fn aumid(&self) -> Result<String> {
        let mut propstore_ptr = std::ptr::null_mut();
        unsafe {
            SHGetPropertyStoreForWindow(self.hwnd, &IID_IPropertyStore, &mut propstore_ptr)
                .context("get property store for window")?
        };
        let propstore = unsafe { IPropertyStore::from_raw(propstore_ptr) };
        let propvar = unsafe {
            propstore
                .GetValue(&PKEY_AppUserModel_ID)
                .context("get aumid property value")?
        };
        let aumid_alloc = unsafe {
            PropVariantToStringAlloc(&propvar).context("get aumid prop variant as string")?
        };
        let aumid = String::from_utf16_lossy(unsafe { aumid_alloc.as_wide() });
        unsafe { CoTaskMemFree(Some(aumid_alloc.as_ptr().cast())) };
        Ok(aumid)
    }
}
