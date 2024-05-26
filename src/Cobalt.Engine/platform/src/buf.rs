use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use windows::Win32::Foundation::UNICODE_STRING;

/// Base trait for buffers meant to be ffi-safe
pub trait Buffer<T> {
    /// Main function to implement. Returns a mutable slice of the buffer.
    fn as_bytes(&mut self) -> &mut [T];

    // reasonable default implementations

    /// Get a pointer to the buffer (we lose size information)
    fn as_mut_ptr(&mut self) -> *mut T {
        self.as_bytes().as_mut_ptr()
    }

    /// Get a pointer to the buffer as a *mut [`c_void`] (we lose size information)
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

/// Trait for buffers that contain wide characters
pub trait WideBuffer: Buffer<u16> {
    // reasonable default implementations

    // use std::ffi::OsString;
    // use windows::core::PWSTR;

    // fn as_pwstr(&mut self) -> PWSTR {
    //     PWSTR(self.as_mut_ptr())
    // }

    // fn to_os_string(&mut self) -> OsString {
    //     use std::os::windows::prelude::OsStringExt;

    //     OsString::from_wide(self.as_bytes())
    // }

    /// Convert this [`WideBuffer`] to a [`String`] using [`String::from_utf16_lossy`].
    /// Notably, invalid data is replaced with the replacement character (U+FFFD).
    fn to_string_lossy(&mut self) -> String {
        String::from_utf16_lossy(self.as_bytes())
    }
}

impl<T: Buffer<u16>> WideBuffer for T {}

/// Buffer that constrains the length of another buffer
pub struct WithLength<T, B: Buffer<T>> {
    _t: PhantomData<T>,
    inner: B,
    length: usize,
}

impl<T, B: Buffer<T>> WithLength<T, B> {
    /// Create a new [`WithLength`] buffer from an inner buffer and a length
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

/// Maximum size for a buffer to be allocated on the stack
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
    /// Create a new [`SmartBuf`] with a given length. Chooses to do a heap allocation
    /// if the length is above N and uses the stack otherwise. Stack allocation is free
    /// anyway, it's only inefficient if its being moved a lot.
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

/// Create a new [`Buffer`] with a given length
pub fn buf<T>(len: usize) -> SmartBuf<T> {
    SmartBuf::new(len)
}
