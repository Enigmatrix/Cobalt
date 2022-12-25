use std::{ffi::OsString, mem::MaybeUninit};

use windows::{core::PWSTR, Win32::Foundation::UNICODE_STRING};

/// Base trait for buffers meant to be ffi-safe
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

    fn to_string_lossy(&mut self) -> String {
        String::from_utf16_lossy(self.as_bytes())
    }

    /// Constrain the length of this Buffer
    fn with_length(self, len: usize) -> WithLength<Self>
    where
        Self: Sized,
    {
        WithLength::new(self, len)
    }
}

pub struct WithLength<B: Buffer> {
    inner: B,
    length: usize,
}

impl<B: Buffer> WithLength<B> {
    pub fn new(inner: B, length: usize) -> WithLength<B> {
        WithLength { inner, length }
    }
}

impl<B: Buffer> Buffer for WithLength<B> {
    fn as_bytes(&mut self) -> &mut [u16] {
        return &mut self.inner.as_bytes()[..self.length];
    }
}

impl Buffer for UNICODE_STRING {
    fn as_bytes(&mut self) -> &mut [u16] {
        unsafe { std::slice::from_raw_parts_mut(self.Buffer.0, (self.Length / 2) as usize) }
    }
}

const SMALL_BUF_STACK_MAX: usize = 0x100;

/// Buffer allocated on the stack if below a certain size, and heap otherwise
pub struct SmallBuf {
    stack: [MaybeUninit<u16>; SMALL_BUF_STACK_MAX],
    heap: Box<[MaybeUninit<u16>]>,
    len: usize,
}

impl SmallBuf {
    pub fn new(len: usize) -> Self {
        Self {
            stack: MaybeUninit::uninit_array(),
            heap: Box::new_uninit_slice(len),
            len,
        }
    }
}

impl Buffer for SmallBuf {
    fn as_bytes(&mut self) -> &mut [u16] {
        let slice = if self.len < SMALL_BUF_STACK_MAX {
            unsafe { MaybeUninit::slice_assume_init_mut(&mut self.stack) }
        } else {
            unsafe { MaybeUninit::slice_assume_init_mut(&mut self.heap) }
        };
        &mut slice[..self.len]
    }
}

/// Create a new buffer allocated on the heap
pub fn buf(len: usize) -> impl Buffer {
    SmallBuf::new(len)
}
