use std::ffi::OsString;
use std::mem::MaybeUninit;
use std::slice;

pub type WBYTE = u16;

pub trait Buffer {
    fn as_bytes(&mut self) -> &mut [WBYTE];

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        self.as_bytes().as_mut_ptr()
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

pub fn alloc(len: usize) -> impl Buffer {
    Alloc {
        inner: Box::new_uninit_slice(len),
    }
}

pub struct Ptr {
    inner: *mut WBYTE,
}

impl Buffer for Ptr {
    fn as_bytes(&mut self) -> &mut [WBYTE] {
        let mut len = 0;
        loop {
            if unsafe { *self.inner.add(len) == 0 } {
                break;
            }
            len += 1;
        }
        unsafe { slice::from_raw_parts_mut(self.inner, len) }
    }

    fn as_mut_ptr(&mut self) -> *mut WBYTE {
        self.inner
    }
}

pub fn from_ptr(ptr: *mut WBYTE) -> impl Buffer {
    Ptr { inner: ptr }
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

pub fn local<const N: usize>() -> Local<N> {
    Local::<N> {
        inner: MaybeUninit::uninit_array::<N>(),
    }
}
