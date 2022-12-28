use std::ffi::c_void;

use utils::errors::*;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{
    CreateTimerQueueTimer, DeleteTimerQueueTimer, WAITORTIMERCALLBACK, WORKER_THREAD_FLAGS,
};

use super::Duration;
use crate::win32;

pub struct Timer {
    handle: HANDLE,
}

impl Timer {
    pub fn new(
        due: Duration,
        period: Duration,
        cb: WAITORTIMERCALLBACK,
        context: Option<*const c_void>,
    ) -> Result<Timer> {
        let mut handle = HANDLE::default();

        win32!(non_zero:
            unsafe {
            CreateTimerQueueTimer(
                &mut handle,
                HANDLE::default(),
                cb,
                context,
                due.millis(),
                period.millis(),
                WORKER_THREAD_FLAGS::default(), // use different flags to change priority
            )
        })
        .context("create native timer")?;

        Ok(Timer { handle })
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        win32!(non_zero: unsafe {
            // INVALID handle for completionevent means that we are waiting for this deletion to finish execution (esp when we have a running callback in timer)
            DeleteTimerQueueTimer(HANDLE::default(), self.handle, HANDLE(-1))
        })
        .expect("drop native timer");
    }
}
