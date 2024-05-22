use std::ffi::c_void;

use windows::Win32::Foundation::{BOOLEAN, HANDLE};
use windows::Win32::System::Threading::{
    CreateTimerQueueTimer, DeleteTimerQueueTimer, WORKER_THREAD_FLAGS,
};

use crate::objects::Duration;

use util::error::*;

/// Win32-based [Timer]. Needs to be scheduled onto a [EventLoop].
pub struct Timer {
    handle: HANDLE,
}

impl Timer {
    /// Create a new [Timer] which calls the callback with the specified due and period.
    /// ### Safety: callback must last as long as the [Timer], and the callback must not be moved.
    pub fn new<F: FnMut() -> Result<()>>(
        due: Duration,
        period: Duration,
        cb: &mut F,
    ) -> Result<Timer> {
        let mut handle = HANDLE::default();

        unsafe {
            CreateTimerQueueTimer(
                &mut handle,
                HANDLE::default(),
                Some(Self::trampoline::<F>),
                Some(cb as *mut _ as *mut _),
                due.millis(),
                period.millis(),
                WORKER_THREAD_FLAGS::default(), // use different flags to change priority
            )
        }
        .context("create native timer")?;

        Ok(Timer { handle })
    }

    unsafe extern "system" fn trampoline<F: FnMut() -> Result<()>>(ctx: *mut c_void, _: BOOLEAN) {
        ctx.cast::<F>().as_mut().unwrap()()
            .context("timer callback")
            .unwrap();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            // INVALID handle for completionevent means that we are waiting for
            // this deletion to finish execution (esp when we have a running callback in timer)
            DeleteTimerQueueTimer(HANDLE(0), self.handle, HANDLE(-1))
        }
        .expect("drop timer");
    }
}
