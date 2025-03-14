use std::hash::{Hash, Hasher};

use util::error::{Context, Result};
use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    GetDesktopWindow, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE, LWA_ALPHA,
    WS_EX_LAYERED,
};

use crate::buf::{buf, Buffer, WideBuffer};
use crate::error::Win32Error;
use crate::objects::ProcessId;
use crate::win32;

/*
 * For the same window, HWND values are the same. If a window is destroyed,
 * its HWND value is no longer valid. Specifically, the value could be reused.
 *
 * We should really be checking if a window is still valid before using it.
 */

/// Representation of a [`Window`] on the user's desktop
#[derive(Clone, PartialEq, Eq)]
pub struct Window {
    hwnd: HWND,
}

impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({:x})", self.hwnd.0)
    }
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

    /// Get the foreground [Window] shown on the user's desktop.
    /// If there is no foreground window or if the window is losing focus, this will return `None`.
    pub fn foreground() -> Option<Self> {
        let fg = unsafe { GetForegroundWindow() };
        if fg == HWND::default() {
            None
        } else {
            Some(Self::new(fg))
        }
    }

    /// Get the Desktop [Window]
    pub fn desktop() -> Self {
        let hwnd = unsafe { GetDesktopWindow() };
        Self::new(hwnd)
    }

    /// Get the [ProcessId] of the [Window]
    pub fn pid(&self) -> Result<ProcessId> {
        let mut pid = 0;
        let _tid = win32!(val: unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) })?;
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

    /// Dim the [Window] by setting its opacity
    pub fn dim(&self, opacity: f64) -> Result<()> {
        // WS_EX_LAYERED will fail for windows with class style of CS_OWNDC or CS_CLASSDC.
        unsafe { SetWindowLongW(self.hwnd, GWL_EXSTYLE, WS_EX_LAYERED.0 as i32) };
        unsafe {
            SetLayeredWindowAttributes(
                self.hwnd,
                COLORREF(0),
                (255.0f64 * opacity) as u8,
                LWA_ALPHA,
            )?
        };
        Ok(())
    }
}
