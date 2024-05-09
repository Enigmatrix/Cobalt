use std::ffi::{c_void, OsString};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use windows::core::PWSTR;
use windows::Win32::Foundation::UNICODE_STRING;

/// Base trait for buffers meant to be ffi-safe
pub trait Buffer<T> {
    // main function to implement
    fn as_bytes(&mut self) -> &mut [T];

    // reasonable default implementations

    fn as_mut_ptr(&mut self) -> *mut T {
        self.as_bytes().as_mut_ptr()
    }

    fn as_mut_void(&mut self) -> *mut c_void {
        self.as_mut_ptr().cast()
    }

    /// Constrain the length of this Buffer
    fn with_length(self, len: usize) -> WithLength<T, Self>
    where
        Self: Sized,
    {
        WithLength::new(self, len)
    }
}

pub trait WideBuffer: Buffer<u16> {
    // reasonable default implementations
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
}

impl<T: Buffer<u16>> WideBuffer for T {}

pub struct WithLength<T, B: Buffer<T>> {
    _t: PhantomData<T>,
    inner: B,
    length: usize,
}

impl<T, B: Buffer<T>> WithLength<T, B> {
    pub fn new(inner: B, length: usize) -> WithLength<T, B> {
        WithLength {
            inner,
            length,
            _t: PhantomData,
        }
    }
}

impl<T, B: Buffer<T>> Buffer<T> for WithLength<T, B> {
    fn as_bytes(&mut self) -> &mut [T] {
        return &mut self.inner.as_bytes()[..self.length];
    }
}

impl Buffer<u16> for UNICODE_STRING {
    fn as_bytes(&mut self) -> &mut [u16] {
        unsafe { std::slice::from_raw_parts_mut(self.Buffer.0, (self.Length / 2) as usize) }
    }
}

pub const SMART_BUF_STACK_MAX: usize = 0x200;

/// Buffer allocated on the stack if below a certain size, and heap otherwise
pub enum SmartBuf<T, const N: usize = SMART_BUF_STACK_MAX> {
    Stack {
        stack: [MaybeUninit<T>; N],
        len: usize,
    },
    Heap {
        heap: Box<[MaybeUninit<T>]>,
    },
}

impl<T, const N: usize> SmartBuf<T, N> {
    pub fn new(len: usize) -> Self {
        if len <= N {
            SmartBuf::Stack {
                stack: MaybeUninit::uninit_array(),
                len,
            }
        } else {
            SmartBuf::Heap {
                heap: Box::new_uninit_slice(len),
            }
        }
    }
}

impl<T, const N: usize> Buffer<T> for SmartBuf<T, N> {
    fn as_bytes(&mut self) -> &mut [T] {
        let uninit_buf = match self {
            SmartBuf::Stack { stack, len } => &mut stack[..*len],
            SmartBuf::Heap { heap } => &mut heap[..],
        };
        unsafe { MaybeUninit::slice_assume_init_mut(uninit_buf) }
    }
}

pub fn buf<T>(len: usize) -> SmartBuf<T> {
    SmartBuf::new(len)
}
