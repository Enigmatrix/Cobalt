use std::hash::{Hash, Hasher};
use std::mem::{self, size_of, MaybeUninit};
use std::ptr;

use windows::core::{ComInterface, Interface};
use windows::imp::{CoTaskMemFree, GetProcAddress, LoadLibraryA};
use windows::s;
use windows::Win32::Foundation::{
    BOOLEAN, COLORREF, HWND, NTSTATUS, RECT, STATUS_BUFFER_TOO_SMALL,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::IsRectEmpty;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::StationsAndDesktops::HDESK;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, PropVariantToStringAlloc, SHGetPropertyStoreForWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongA, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, SetLayeredWindowAttributes, SetWindowLongA,
    GWL_EXSTYLE, LWA_ALPHA, WS_EX_LAYERED, GWL_STYLE, WS_EX_TRANSPARENT, WS_EX_TOOLWINDOW,
};

use crate::objects::process::ProcessId;
use crate::*;

use common::errors::*;

/// Representation of a Window on the user's desktop
#[derive(Clone, PartialEq, Eq)]
pub struct Window {
    inner: HWND,
}

impl Hash for Window {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.0.hash(state);
    }
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
        // TODO just call GetWindowTextW first, forget the call to GetWindowTextLength until it fails
        // important for some reason ...
        Win32Error::clear_last_err();
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

    /// Checks if the [Window] is visible to the user
    pub fn visible(&self) -> Result<bool> {
        // Check if window has the WS_VISIBLE property
        if unsafe { !IsWindowVisible(self.inner).as_bool() } {
            return Ok(false)
        }
        
        let exstyle = unsafe { GetWindowLongA(self.inner, GWL_EXSTYLE) } as u32;
        // Check for the "transparent" windows, where hit-testing falls through them.
        // These are most likely not real windows, but rather overlays.
        if exstyle & WS_EX_TRANSPARENT.0 != 0 {
            return Ok(false)
        }
        // Check for the tool windows, which are floating windows
        // These are most likely not real windows, but rather overlays.
        if exstyle & WS_EX_TOOLWINDOW.0 != 0 {
            return Ok(false)
        }

        // Check if window isn't cloaked (Windows 10)
        let mut cloaked: u32 = 0;
        unsafe {
            DwmGetWindowAttribute(
                self.inner,
                DWMWA_CLOAKED,
                &mut cloaked as *mut _ as *mut _,
                size_of::<u32>() as u32,
            )?
        };
        if cloaked != 0 {
            return Ok(false)
        }

        // Check if window has a empty area
        let mut rect = RECT::default();
        unsafe { GetWindowRect(self.inner, &mut rect).ok()? }
        if unsafe { IsRectEmpty(&rect).as_bool() } {
            return Ok(false)
        }

        Ok(true)
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

    /// Set the WS_EX_LAYERED style on this [Window]
    pub fn enable_layered(&self) -> Result<()> {
        loop {
            let style = win32!(val: unsafe { GetWindowLongA(self.inner, GWL_EXSTYLE) })?;
            let old_style = win32!(val: unsafe { SetWindowLongA(self.inner, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as i32) })?;
            if style == old_style {
                break;
            }
        }
        Ok(())
    }

    /// Change the layer opacity for this [Window]. Assumes the [Window] is already a layered window.
    pub fn layer_dim(&self, alpha: u8) -> Result<()> {
        // TODO check https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-updatelayeredwindow, that might be able to give black bg
        unsafe { SetLayeredWindowAttributes(self.inner, COLORREF(0), alpha, LWA_ALPHA).ok()? };
        Ok(())
    }

    /// Gets all the [Window] on the user's Desktop. Note that this include invisible windows.
    pub fn get_all() -> Result<impl Iterator<Item = Window>> {
        // let windows = Vec::new();
        let mut needed = 128;
        loop {
            let sz = needed as usize + 128; // extra, in case there are more windows spawned between the two calls
            let mut buf = buffers::buf::<HWND>(sz);
            let err = unsafe {
                NtUserBuildHwndList.assume_init()(
                    HDESK(0),
                    HWND(0),
                    true.into(),
                    false.into(),
                    0,
                    sz as u32,
                    buf.as_mut_ptr(),
                    &mut needed,
                )
            };
            if err != STATUS_BUFFER_TOO_SMALL {
                return NtError::from(err)
                    .to_result()
                    .map(|_| {
                        unsafe { std::slice::from_raw_parts(buf.as_mut_ptr(), sz) }
                            .iter()
                            .cloned()
                            .map(Window::new)
                    })
                    .context("build hwnd list");
            }
        }
    }

    /// Sets up native functions used by [Window]
    pub(crate) fn setup() -> Result<()> {
        let win32u = win32!(val: unsafe { LoadLibraryA(s!("win32u")) })?;
        unsafe {
            NtUserBuildHwndList = MaybeUninit::new(mem::transmute(
                win32!(ptr: GetProcAddress(win32u, s!("NtUserBuildHwndList")) )?,
            ))
        };
        Ok(())
    }
}

#[allow(non_upper_case_globals)]
static mut NtUserBuildHwndList: MaybeUninit<
    fn(
        hDesktop: HDESK,
        hwndParent: HWND,
        bChildren: BOOLEAN,
        hideImmersive: BOOLEAN,
        dwThreadId: u32,
        cHwnd: u32,
        phwndList: *mut HWND,
        pcHwndNeeded: *mut u32,
    ) -> NTSTATUS,
> = MaybeUninit::uninit();
