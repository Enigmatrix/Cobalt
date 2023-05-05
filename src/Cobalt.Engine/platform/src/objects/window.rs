use std::ptr;

use windows::core::{ComInterface, Interface};
use windows::imp::CoTaskMemFree;
use windows::Win32::Foundation::HWND;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, PropVariantToStringAlloc, SHGetPropertyStoreForWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

use crate::objects::process::ProcessId;
use crate::*;

use common::errors::*;

/// Representation of a Window on the user's desktop
#[derive(Clone, PartialEq, Eq)]
pub struct Window {
    inner: HWND,
}

impl Window {
    /// Create a new [Window]
    pub fn new(hwnd: HWND) -> Self {
        Self { inner: hwnd }
    }

    /// Get the foreground [Window] shown on the user's desktop
    pub fn foreground() -> Option<Self> {
        let fg = unsafe { GetForegroundWindow() };
        if fg == HWND::default() {
            None
        } else {
            Some(Self::new(fg))
        }
    }

    /// Get the [ProcessId] of the [Window]
    pub fn pid(&self) -> Result<ProcessId> {
        let mut pid = 0;
        win32!(val: unsafe { GetWindowThreadProcessId(self.inner, Some(&mut pid)) })
            .context("get window pid")?;
        Ok(pid)
    }

    /// Get the title of the [Window]
    pub fn title(&self) -> Result<String> {
        let len = unsafe { GetWindowTextLengthW(self.inner) };

        let title = if len == 0 {
            Win32Error::last_result()
                .map(|_| String::new())
                .context("get window title length")?
        } else {
            let mut buf = buffers::buf(len as usize + 1);
            let written = win32!(val: unsafe { GetWindowTextW(self.inner, buf.as_bytes()) })
                .context("get window title")?;
            buf.with_length(written as usize).to_string_lossy()
        };
        Ok(title)
    }

    /// Get the [AUMID](https://learn.microsoft.com/en-us/windows/win32/shell/appids) of the [Window]
    pub fn aumid(&self) -> Result<String> {
        let mut prop_store_raw = ptr::null_mut();
        unsafe {
            SHGetPropertyStoreForWindow(
                self.inner,
                &<IPropertyStore as ComInterface>::IID,
                &mut prop_store_raw,
            )
            .context("get window property store")?;
        };

        let prop_store = unsafe { IPropertyStore::from_raw(prop_store_raw) };

        let aumid_prop = unsafe {
            prop_store
                .GetValue(&PKEY_AppUserModel_ID)
                .context("get window aumid property")?
        };

        let aumid_raw = unsafe {
            PropVariantToStringAlloc(&aumid_prop)
                .context("get aumid property as allocated string")?
        };

        let aumid = String::from_utf16_lossy(unsafe { aumid_raw.as_wide() });
        unsafe { CoTaskMemFree(aumid_raw.as_ptr().cast()) };

        Ok(aumid)
    }
}
