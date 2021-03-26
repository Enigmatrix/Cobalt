use crate::buffer::{self, Buffer};
use crate::error::*;
use crate::raw::*;
use crate::wrappers::process::*;
use std::default::default;
use std::hash;
use std::ptr;
use std::fmt;
use util::*;

#[derive(Clone)]
pub struct Window(pub HWND);
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("hwnd", &self.0)
            .field("title", &self.title())
            .field("class", &self.class())
            .finish()
    }
}

impl PartialEq<Window> for Window {
    fn eq(&self, other: &Window) -> bool {
        self.0 == other.0
    }
}

impl Eq for Window {}

impl PartialEq<HWND> for Window {
    fn eq(&self, other: &HWND) -> bool {
        self.0 == *other
    }
}

impl hash::Hash for Window {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: hash::Hasher,
    {
        hasher.write_usize(self.0 as usize);
    }
}

impl Window {
    pub fn new(handle: HWND) -> Result<Window, Win32Err> {
        Ok(Window(handle))
    }

    pub fn foreground() -> Result<Window, Win32Err> {
        Window::new(win32!(non_null: winuser::GetForegroundWindow())?)
    }

    pub fn title(&self) -> Result<String, Win32Err> {
        Win32Err::clear_last_err(); // yes, actually important!
        let len = unsafe { winuser::GetWindowTextLengthW(self.0) };
        // fails if len == 0 && !Win32Code::from_last().is_success()
        if len == 0 {
            let err = Win32Err::last_err();
            if err.is_success() {
                Ok(String::new())
            } else {
                Err(err)
            }
        } else {
            let mut buf = buffer::alloc(len as usize + 1);
            let written =
                win32!(non_zero: winuser::GetWindowTextW(self.0, buf.as_mut_ptr(), len + 1))?;
            Ok(buf.with_length(written as usize).as_string_lossy())
        }
    }

    pub fn class(&self) -> Result<String, Win32Err> {
        let mut buf_len: i32 = 256;
        let mut buf = buffer::alloc(buf_len as usize);
        loop {
            let read = unsafe { winuser::GetClassNameW(self.0, buf.as_mut_ptr(), buf_len) };
            if read == 0 {
                let err = Win32Err::last_err();
                if err.is_success() {
                    buf_len *= 2;
                    buf = buffer::alloc(buf_len as usize);
                    continue
                }
                else {
                    return Err(err);
                }
            }
            else {
                buf_len = read;
                break;
            }

        }
        Ok(buf.with_length((buf_len) as usize).as_string_lossy())
    }

    pub fn pid_tid(&self) -> Result<(ProcessId, u32), Win32Err> {
        let mut pid = 0;
        let tid = unsafe { winuser::GetWindowThreadProcessId(self.0, &mut pid) };
        if pid == 0 || tid == 0 {
            Err(Win32Err::last_err())
        } else {
            Ok((pid, tid))
        }
    }

    pub fn is_uwp(&self, process: &Process, path: &str) -> bool {
        (unsafe { winuser::IsImmersiveProcess(process.handle()) != 0 })
            && (path.eq_ignore_ascii_case("C:\\Windows\\System32\\ApplicationFrameHost.exe"))
    }

    pub fn aumid(&self) -> Result<String> {
        let mut property_store: *mut propsys::IPropertyStore = ptr::null_mut();
        hresult!({
            shellapi::SHGetPropertyStoreForWindow(
                self.0,
                &uuid::IID_IPropertyStore as *const _ as *const _,
                &mut property_store as *mut _ as *mut *mut _,
            )
        })?;

        let mut prop: propidl::PROPVARIANT = default();
        hresult!((*property_store).GetValue(&propkey::PKEY_AppUserModel_ID as *const _, &mut prop))?;

        let aumid_ptr = unsafe { *prop.data.pwszVal() };
        if aumid_ptr.is_null() {
            Err(anyhow!("AUMID is null or not a string"))
        } else {
            Ok(buffer::from_ptr(aumid_ptr).as_string_lossy())
        }
    }
}
