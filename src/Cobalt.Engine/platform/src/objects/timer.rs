use std::ffi::c_void;

use util::error::*;
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::{BOOLEAN, HANDLE};
use windows::Win32::System::Threading::{
    CreateTimerQueueTimer, DeleteTimerQueueTimer, WORKER_THREAD_FLAGS,
};

use crate::objects::Duration;

type Callback = Box<dyn FnMut() -> Result<()>>;

/// Win32-based [Timer]. Needs to be scheduled onto a [EventLoop].
pub struct Timer {
    handle: HANDLE,
    _cb: Box<Callback>,
}

impl Timer {
    /// Create a new [Timer] which calls the callback with the specified due and period.
    pub fn new(due: Duration, period: Duration, cb: Callback) -> Result<Timer> {
        let mut handle = HANDLE::default();
        // Need to double box (Callback is also boxed) - cb.as_mut() on Callback
        // will return a vtable+data rather than just one pointer. Double boxing
        // will give us a single pointer but we pay for an additional indirection.
        let mut cb = Box::new(cb);

        unsafe {
            CreateTimerQueueTimer(
                &mut handle,
                HANDLE::default(),
                Some(Self::trampoline),
                Some(cb.as_mut() as *mut _ as *mut _),
                due.millis(),
                period.millis(),
                WORKER_THREAD_FLAGS::default(), // use different flags to change priority
            )
        }
        .context("create native timer")?;

        Ok(Timer { handle, _cb: cb })
    }

    unsafe extern "system" fn trampoline(ctx: *mut c_void, _: BOOLEAN) {
        ctx.cast::<Callback>().as_mut().unwrap()()
            .context("timer callback")
            .error();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            // INVALID handle for completionevent means that we are waiting for
            // this deletion to finish execution (esp when we have a running callback in timer)
            DeleteTimerQueueTimer(HANDLE(0), self.handle, HANDLE(-1))
        }
        .context("drop timer")
        .warn();
    }
}
