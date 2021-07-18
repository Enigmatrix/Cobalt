use std::{ffi::c_void, marker::PhantomData};
use bindings::Windows::Win32::{Foundation::HANDLE, System::Threading::{CreateTimerQueueTimer, DeleteTimerQueueTimer, WORKER_THREAD_FLAGS}};

pub struct Timer<F> where F: FnMut() {
    handle: HANDLE,
    _cb: PhantomData<F>,
}

impl<F: FnMut()> Timer<F> {
    pub fn new(due: u32, period: u32, cb: &mut F) -> Timer<F> {
        let mut handle = HANDLE::NULL;

        // TODO win32 wrapper
        unsafe {
            CreateTimerQueueTimer(
                &mut handle,
                HANDLE::NULL,
                Some(Timer::<F>::_cb),
                cb as *mut _ as *mut _,
                due,
                period,
                WORKER_THREAD_FLAGS::from(0)) // we use the default flags, use different ones if performance profile is insufficient
        };

        Timer { handle, _cb: PhantomData }
    }

    unsafe extern "system" fn _cb(cb: *mut c_void, _: u8) {
        let cb = cb.cast::<F>().as_mut();
        // TODO use error-handling instead of expect
        // TODO wrap in error-handling checker, from util
        cb.expect("Callback point to be non-null")();
    }
}

impl<F: FnMut()> Drop for Timer<F> {
    fn drop(&mut self) {
        unsafe {
            // INVALID handle for completionevent means that we are waiting for this deletion to finish execution (esp when we have a running callback in timer)
            DeleteTimerQueueTimer(HANDLE::NULL, self.handle, HANDLE::INVALID);
        }
    }
}