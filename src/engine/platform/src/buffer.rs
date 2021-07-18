use std::{ffi::OsString, mem::MaybeUninit};
use bindings::Windows::Win32::Foundation::PWSTR;

type WCHAR = u16;

#[doc = "Base trait for buffers meant to be ffi-safe"]
trait Buffer {
    // main function to implement
    fn as_bytes(&mut self) -> &mut [WCHAR];


    // reasonable default implementations
    fn as_mut_ptr(&mut self) -> *mut WCHAR {
        self.as_bytes().as_mut_ptr()
    }

    fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.as_mut_ptr())
    }

    fn to_os_string(&mut self) -> OsString {
        use std::os::windows::prelude::OsStringExt;

        OsString::from_wide(self.as_bytes())
    }
}

#[doc = "Buffer allocated on the heap"]
pub struct Alloc {
    inner: Box<[MaybeUninit<WCHAR>]>,
}

impl Alloc {
    pub fn new(len: usize) -> Alloc {
        Alloc { inner: Box::new_uninit_slice(len) }
    }
}

impl Buffer for Alloc {
    fn as_bytes(&mut self) -> &mut [WCHAR] {
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.inner) }
    }

    fn as_mut_ptr(&mut self) -> *mut WCHAR {
        MaybeUninit::slice_as_mut_ptr(&mut self.inner)
    }
}

#[doc = "Create a new buffer allocated on the heap"]
pub fn alloc(len: usize) -> Alloc {
    Alloc::new(len)
}