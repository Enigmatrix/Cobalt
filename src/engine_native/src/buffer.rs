use std::{ffi::OsString, fmt::{self, write}, mem::{MaybeUninit}};

use engine_windows_bindings::windows::win32::system_services::PWSTR;

pub type WBYTE = u16;

pub trait Buffer {
    fn as_bytes(&mut self) -> &mut [WBYTE];

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        self.as_bytes().as_mut_ptr()
    }

    fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.as_mut_ptr())
    }

    fn as_os_string(&mut self) -> OsString {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(self.as_bytes())
    }

    fn as_string(&mut self) -> Result<String, OsString> {
        self.as_os_string().into_string()
    }

    fn as_string_lossy(&mut self) -> String {
        self.as_os_string().to_string_lossy().to_string()
    }

    fn with_length(&mut self, len: usize) -> WithLength<Self>
    where
        Self: Sized,
    {
        WithLength { inner: self, len }
    }
}

impl fmt::Debug for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string_lossy())
    }
}

pub struct WithLength<'a, T: Buffer> {
    inner: &'a mut T,
    len: usize,
}

impl<'a, T: Buffer> Buffer for WithLength<'a, T> {
    fn as_bytes(&mut self) -> &mut [WBYTE] {
        &mut self.inner.as_bytes()[..self.len]
    }
}

pub struct Alloc {
    inner: Box<[MaybeUninit<WBYTE>]>,
}

impl Buffer for Alloc {
    fn as_bytes(&mut self) -> &mut [WBYTE] {
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.inner) }
    }

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        MaybeUninit::slice_as_mut_ptr(&mut self.inner)
    }
}

pub struct Local<const N: usize> {
    inner: [MaybeUninit<WBYTE>; N],
}

impl<const N: usize> Buffer for Local<N> {
    fn as_bytes(&mut self) -> &mut [WBYTE] {
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.inner) }
    }

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        MaybeUninit::slice_as_mut_ptr(&mut self.inner)
    }
}

pub enum Flexible<const N: usize> {
    Local(Local<N>),
    Alloc(Alloc)
}

impl<const N: usize> Flexible<N> {
    pub fn resized_to(mut self, len: usize) -> Flexible<N> {
        if len > self.as_bytes().len() {
            Flexible::Alloc(alloc(len))
        } else {
            self
        }
    }

    pub fn resized(mut self) -> Flexible<N> {
        let ns = self.as_bytes().len() * 2;
        self.resized_to(ns)
    }
}

impl<const N: usize> Buffer for Flexible<N> {
    fn as_bytes(&mut self) -> &mut [WBYTE] {
        match self {
            Flexible::Local(local) => local.as_bytes(),
            Flexible::Alloc(alloc) => alloc.as_bytes()
        }
    }

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        match self {
            Flexible::Local(local) => local.as_mut_ptr(),
            Flexible::Alloc(alloc) => alloc.as_mut_ptr()
        }
    }
}

pub fn alloc(len: usize) -> Alloc {
    Alloc {
        inner: Box::new_uninit_slice(len),
    }
}

pub fn local<const N: usize>() -> Local<N> {
    Local::<N> {
        inner: MaybeUninit::uninit_array::<N>(),
    }
}

pub fn flexible<const N: usize>() -> Flexible<N> {
    Flexible::Local(local::<N>())
}