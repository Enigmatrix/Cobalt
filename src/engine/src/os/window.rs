use crate::os::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Window(HWND);

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

impl std::hash::Hash for Window {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: std::hash::Hasher,
    {
        hasher.write_usize(self.0 as usize);
    }
}

impl Window {
    pub fn new(handle: HWND) -> Result<Window, crate::os::error::Error> {
        Ok(Window(handle))
    }

    pub fn title(&self) -> Result<String, crate::os::error::Error> {
        // TODO String or WideString (os::String)
        let len = unsafe { winuser::GetWindowTextLengthW(self.0) };
        // fails if len == 0 && !Error::last_win32().successful()
        if len == 0 {
            let err = Error::last_win32();
            if err.successful() {
                Ok(String::new())
            } else {
                Err(err)
            }
        } else {
            let mut buf = string_buffer!(len + 1);
            let written =
                expect!(true: winuser::GetWindowTextW(self.0, buf.as_mut_ptr(), len + 1))? as usize;
            Ok(string_from_buffer!(buf, written))
        }
    }

    pub fn pid_tid(&self) -> Result<(u32, u32), crate::os::error::Error> {
        let mut pid = 0;
        let tid = unsafe { winuser::GetWindowThreadProcessId(self.0, &mut pid) };
        if pid == 0 || tid == 0 {
            Err(Error::last_win32())
        } else {
            Ok((pid, tid))
        }
    }

    pub fn is_uwp(&self) -> Result<bool, crate::os::error::Error> {
        let (pid, _) = self.pid_tid()?;
        let proc = Process::new(pid, default())?;
        if unsafe { winuser::IsImmersiveProcess(proc.handle()) } != 0 {
            let path = proc.path_fast()?;
            Ok(path.eq_ignore_ascii_case("C:\\Windows\\System32\\ApplicationFrameHost.exe"))
        } else {
            Ok(false)
        }
    }

    pub fn aumid(&self) -> Result<String, crate::os::error::Error> {
        let mut property_store: *mut propsys::IPropertyStore = ptr::null_mut();
        hresult!({
            shellapi::SHGetPropertyStoreForWindow(
                self.0,
                &uuid::IID_IPropertyStore as *const _ as *const _,
                &mut property_store as *mut _ as *mut *mut c_void,
            )
        })?;

        let mut prop: propidl::PROPVARIANT = default();
        hresult!((*property_store).GetValue(&propkey::PKEY_AppUserModel_ID as *const _, &mut prop))?;

        let aumid_ptr = unsafe { *prop.data.pwszVal() }; // TODO check
        let mut aumid_len = 0;
        loop {
            if unsafe { *aumid_ptr.offset(aumid_len) == 0 } {
                break;
            }
            aumid_len += 1;
        }

        Ok(string_from_buffer!(unsafe {
            std::slice::from_raw_parts(aumid_ptr, aumid_len as usize)
        }))
        //Ok(unsafe { ffi::NulString::from_raw(aumid_ptr).to_ustring() })
    }
}
