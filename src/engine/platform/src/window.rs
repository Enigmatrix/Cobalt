use bindings::{Windows::Win32::{Foundation::HWND, System::{Com::CoTaskMemFree, PropertiesSystem::{IPropertyStore, PROPERTYKEY, PropVariantToStringAlloc, SHGetPropertyStoreForWindow}}, UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW}}, meta::Guid};

use std::{ffi::OsString, fmt, hash};

use crate::{buffer::Buffer, buffer, error::Win32Err, win32};

pub struct Window {
    hwnd: HWND
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("hwnd", &self.hwnd)
            .field("title", &self.title())
            .field("aumid", &self.aumid())
            .finish()
    }
}

impl PartialEq<Window> for Window {
    fn eq(&self, other: &Window) -> bool {
        self.hwnd == other.hwnd
    }
}

impl Eq for Window {}

impl PartialEq<HWND> for Window {
    fn eq(&self, other: &HWND) -> bool {
        self.hwnd == *other
    }
}

impl hash::Hash for Window {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: hash::Hasher,
    {
        hasher.write_usize(self.hwnd.0 as usize);
    }
}

#[allow(non_upper_case_globals)]
pub static PKEY_AppUserModel_ID: PROPERTYKEY = PROPERTYKEY { fmtid: Guid::from_values(0x9F4C2855, 0x9F79, 0x4B39, [0xA8, 0xD0, 0xE1, 0xD4, 0x2D, 0xE1, 0xD5, 0xF3]), pid: 5 };

impl Window {
    pub fn new(hwnd: HWND) -> Window {
        Window { hwnd }
    }

    pub fn title(&self) -> Result<OsString, Win32Err> {
        Win32Err::clear_last_err(); // yes, actually important!
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        // fails if len == 0 && !Win32Err::last_err().is_success()
        if len == 0 {
            Win32Err::last_result().map(|_| OsString::new())
        } else {
            let mut buf = buffer::alloc(len as usize + 1);
            let written =
                win32!(non_zero: GetWindowTextW(self.hwnd, buf.as_pwstr(), len + 1))?;

            Ok(buf.with_length(written as usize).to_os_string())
        }
    }

    pub fn foreground() -> Result<Window, Win32Err> {
        Ok(Window::new(win32!(non_zero: inner GetForegroundWindow())?))
    }

    /*
    pub fn is_uwp(&self, process: &Process, path: &str) -> bool {
        (unsafe { winuser::IsImmersiveProcess(process.handle()) != 0 })
            && (path.eq_ignore_ascii_case("C:\\Windows\\System32\\ApplicationFrameHost.exe"))
    }
    */

    pub fn aumid(&self) -> ::bindings::meta::Result<OsString> {
        let propstore: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(self.hwnd)? };
        let propvar = unsafe { propstore.GetValue(&PKEY_AppUserModel_ID)? };
        let ptr = unsafe { PropVariantToStringAlloc(&propvar)?.0 };
        let len = unsafe {
            let mut i = 0;
            while *ptr.add(i) != 0 {
                i += 1;
            }
            i
        };

        let aumid = unsafe {
            use std::os::windows::prelude::OsStringExt;
            OsString::from_wide(std::slice::from_raw_parts_mut(ptr, len))
        };

        unsafe { CoTaskMemFree(ptr as *mut _) };
        
        Ok(aumid)
    }
}