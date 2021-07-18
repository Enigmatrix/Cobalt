use std::{ffi::OsString, mem::MaybeUninit};
use bindings::Windows::Win32::Foundation::PWSTR;

#[doc = "Base trait for buffers meant to be ffi-safe"]
pub trait Buffer {
    // main function to implement
    fn as_bytes(&mut self) -> &mut [u16];


    // reasonable default implementations
    fn as_mut_ptr(&mut self) -> *mut u16 {
        self.as_bytes().as_mut_ptr()
    }

    fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.as_mut_ptr())
    }

    fn to_os_string(&mut self) -> OsString {
        use std::os::windows::prelude::OsStringExt;

        OsString::from_wide(self.as_bytes())
    }

    fn with_length(self, len: usize) -> WithLength<Self> where Self: Sized {
        WithLength::new(self, len)
    }
}

pub struct WithLength<B: Buffer> {
    inner: B,
    length: usize
}

impl<B: Buffer> WithLength<B> {
    pub fn new(inner: B, length: usize) -> WithLength<B> {
        WithLength { inner, length }
    }
}

impl<B: Buffer> Buffer for WithLength<B> {
    fn as_bytes(&mut self) -> &mut [u16] {
        return &mut self.inner.as_bytes()[..self.length]
    }
}


#[doc = "Buffer allocated on the heap"]
pub struct Alloc {
    inner: Box<[MaybeUninit<u16>]>,
}

impl Alloc {
    pub fn new(len: usize) -> Alloc {
        Alloc { inner: Box::new_uninit_slice(len) }
    }
}

impl Buffer for Alloc {
    fn as_bytes(&mut self) -> &mut [u16] {
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.inner) }
    }

    fn as_mut_ptr(&mut self) -> *mut u16 {
        MaybeUninit::slice_as_mut_ptr(&mut self.inner)
    }
}

#[doc = "Create a new buffer allocated on the heap"]
pub fn alloc(len: usize) -> Alloc {
    Alloc::new(len)
}