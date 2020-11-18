use crate::error::HResult;
use crate::raw::unknwnbase::IUnknown;
use crate::raw::*;
use std::fmt;
use std::mem::forget;
use std::ops::Deref;
use std::ptr::{null_mut, NonNull};

#[repr(transparent)]
pub struct Com<T>(NonNull<T>);

impl<T> Com<T> {
    /// # Safety
    /// Ensure `ptr` points to a valid Com object, or NULL
    pub unsafe fn new(ptr: *mut T) -> Option<Com<T>>
    where
        T: Interface,
    {
        NonNull::new(ptr).map(Com)
    }

    /// # Safety
    /// Ensure `ptr` points to a valid Com object
    pub unsafe fn from_raw(ptr: *mut T) -> Com<T>
    where
        T: Interface,
    {
        Com(NonNull::new(ptr).expect("ptr should not be null"))
    }

    pub fn create(clsid: guiddef::CLSID) -> Result<Com<T>, HResult>
    where
        T: Interface,
    {
        Com::<T>::from_fn(|instance| {
            unsafe { combaseapi::CoCreateInstance(
                &clsid,
                std::ptr::null_mut(),
                wtypesbase::CLSCTX_INPROC_SERVER,
                &T::uuidof(),
                instance as *mut _ as *mut _,
            ) }
        })
        .transpose()
        .unwrap()
    }

    pub fn from_fn(
        fun: impl FnOnce(&mut *mut T) -> HRESULT,
    ) -> Result<Option<Com<T>>, HResult>
    where
        T: Interface,
    {
        let mut ptr = null_mut();
        let val = fun(&mut ptr);
        let com = unsafe { Com::new(ptr) };

        if val < 0 {
            if com.is_some() {
                tracing::warn!("ComPtr::from_fn had an initialized COM pointer despite the function returning an error")
            }
            return Err(HResult::new(val));
        } else if val != 0 {
            tracing::warn!("HRESULT WARN 0x{:0x}", val);
        }
        Ok(com)
    }

    pub fn up<U>(self) -> Com<U>
    where
        T: Deref<Target = U>,
        U: Interface,
    {
        unsafe { Com::from_raw(self.into_raw() as *mut U) }
    }

    pub fn into_raw(self) -> *mut T {
        let p = self.0.as_ptr();
        forget(self);
        p
    }

    fn as_unknown(&self) -> &IUnknown {
        unsafe { &*(self.as_raw() as *mut IUnknown) }
    }

    pub fn cast<U>(&self) -> Result<Com<U>, i32>
    where
        U: Interface,
    {
        let mut obj = null_mut();
        let err = unsafe { self.as_unknown().QueryInterface(&U::uuidof(), &mut obj) };
        if err < 0 {
            return Err(err);
        }
        Ok(unsafe { Com::from_raw(obj as *mut U) })
    }

    pub fn as_raw(&self) -> *mut T {
        self.0.as_ptr()
    }
}
impl<T> Deref for Com<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.as_raw() }
    }
}
impl<T> Clone for Com<T>
where
    T: Interface,
{
    fn clone(&self) -> Self {
        unsafe {
            self.as_unknown().AddRef();
            Com::from_raw(self.as_raw())
        }
    }
}
impl<T> fmt::Debug for Com<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
impl<T> Drop for Com<T> {
    fn drop(&mut self) {
        unsafe {
            self.as_unknown().Release();
        }
    }
}
impl<T> PartialEq<Com<T>> for Com<T>
where
    T: Interface,
{
    fn eq(&self, other: &Com<T>) -> bool {
        self.0 == other.0
    }
}
