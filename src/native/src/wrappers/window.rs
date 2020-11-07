use crate::buffer::{self, Buffer};
use crate::error::*;
use crate::raw::*;
use crate::wrappers::process::ProcessId;
use std::default::default;
use std::hash;
use std::ptr;

#[derive(Debug, Clone)]
pub struct Window(pub HWND);
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

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

    pub fn title(&self) -> Result<String, Win32Err> {
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
            Ok(buf.with_length(written as usize).to_string_lossy())
        }
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

    pub fn aumid(&self) -> Result<String, HResult> {
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

        let aumid_ptr = unsafe { *prop.data.pwszVal() }; // TODO check

        Ok(buffer::from_ptr(aumid_ptr).to_string_lossy())
    }
}
