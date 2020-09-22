use crate::os::*;

#[derive(Debug, Clone)]
pub struct Window(HWND);

impl PartialEq<Window> for Window {
    fn eq(&self, other: &Window) -> bool {
        self.0 == other.0
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
    pub fn new(handle: HWND) -> Window {
        Window(handle)
    }

    pub fn title(&self) -> Result<String, crate::os::Error> {
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
            use std::os::windows::ffi::OsStringExt;

            let mut buf = vec![0u16; len as usize];
            unsafe { winuser::GetWindowTextW(self.0, buf.as_mut_ptr(), len + 1) }; // TODO this could error out
            Ok(std::ffi::OsString::from_wide(&buf).to_string_lossy().into())
        }
    }
}
