use std::hash::{Hash, Hasher};

use util::error::{Context, Result};
use windows::Win32::Foundation::{BOOL, COLORREF, HWND, LPARAM};
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GWL_EXSTYLE, GetClassNameW, GetForegroundWindow,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    LWA_ALPHA, SetLayeredWindowAttributes, SetWindowLongW, WS_EX_LAYERED,
};

use crate::buf::{Buffer, WideBuffer, buf};
use crate::error::Win32Error;
use crate::objects::ProcessThreadId;
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
    /// The handle to the window
    pub hwnd: HWND,
}

// Needed because HWND is not Send/Sync - but it really should be
// since it's just a number
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({:x})", self.hwnd.0 as usize)
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

    /// Get the [ProcessThreadId] of the [Window]
    pub fn ptid(&self) -> Result<ProcessThreadId> {
        let mut pid = 0;
        let tid = win32!(val: unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) })?;
        Ok(ProcessThreadId { pid, tid })
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

    /// Get the class name of the [Window]
    pub fn class(&self) -> Result<String> {
        let mut buf = buf(256);
        let written = win32!(val: unsafe { GetClassNameW(self.hwnd, buf.as_bytes()) })?;
        Ok(buf.with_length(written as usize).to_string_lossy())
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

    /// Check if the [Window] is minimized
    pub fn is_minimized(&self) -> bool {
        unsafe { IsIconic(self.hwnd).as_bool() }
    }

    /// Get all visible windows on the desktop
    pub fn get_all_visible_windows() -> Result<Vec<Self>> {
        let mut windows = Vec::new();
        unsafe {
            EnumWindows(
                Some(push_visible_window_callback),
                LPARAM(&mut windows as *mut _ as isize),
            )?;
        }
        Ok(windows)
    }

    /// Get all children of the [Window]
    pub fn children(&self) -> Result<Vec<Self>> {
        let mut children = Vec::new();
        unsafe {
            EnumChildWindows(
                Some(self.hwnd),
                Some(push_window_callback),
                LPARAM(&mut children as *mut _ as isize),
            )
            .ok()?;
        }
        Ok(children)
    }
}

unsafe extern "system" fn push_visible_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<Window>) };

    // Only add windows that are visible
    if unsafe { IsWindowVisible(hwnd).as_bool() } {
        windows.push(Window::new(hwnd));
    }

    BOOL(1) // Continue enumeration
}

unsafe extern "system" fn push_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<Window>) };
    windows.push(Window::new(hwnd));

    BOOL(1) // Continue enumeration
}
