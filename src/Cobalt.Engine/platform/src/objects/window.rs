use std::hash::{Hash, Hasher};

use util::error::{Context, Result};
use windows::Win32::{
    Foundation::HWND,
    Storage::EnhancedStorage::PKEY_AppUserModel_ID,
    System::Com::{CoTaskMemFree, StructuredStorage::PropVariantToStringAlloc},
    UI::{
        Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    },
};

use crate::{
    buf::{buf, Buffer, WideBuffer},
    error::Win32Error,
    objects::ProcessId,
    win32,
};

/*
 * For the same window, HWND values are the same. If a window is destroyed,
 * its HWND value is no longer valid. Specifically, the value could be reused.
 *
 * We should really be checking if a window is still valid before using it.
 */

/// Representation of a Window on the user's desktop
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    hwnd: HWND,
}

impl Hash for Window {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hwnd.0.hash(state);
    }
}

impl Window {
    /// Create a new [Window]
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
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
        let _tid = win32!(val: unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) });
        Ok(pid)
    }

    /// Get the title of the [Window]
    pub fn title(&self) -> Result<String> {
        // TODO optim: just call GetWindowTextW first, forget the call to
        // GetWindowTextLength until it fails

        // important for some reason ...
        Win32Error::clear_last_err();
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };

        let title = if len == 0 {
            Win32Error::last_result()
                .map(|_| String::new())
                .context("get window title length")?
        } else {
            let mut buf = buf(len as usize + 1);
            let written = win32!(val: unsafe { GetWindowTextW(self.hwnd, buf.as_bytes()) })?;
            buf.with_length(written as usize).to_string_lossy()
        };
        Ok(title)
    }

    /// Get the [AUMID](https://learn.microsoft.com/en-us/windows/win32/shell/appids) of the [Window]
    pub fn aumid(&self) -> Result<String> {
        let prop_store: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(self.hwnd)? };
        let aumid_prop = unsafe { prop_store.GetValue(&PKEY_AppUserModel_ID)? };

        let aumid_raw = unsafe { PropVariantToStringAlloc(&aumid_prop)? };
        // This is a copy, so we are free to CoTaskMemFree the aumid_raw in the next line
        let aumid = String::from_utf16_lossy(unsafe { aumid_raw.as_wide() });
        unsafe { CoTaskMemFree(Some(aumid_raw.as_ptr().cast())) };

        Ok(aumid)
    }
}
