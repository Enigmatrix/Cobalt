use std::cell::RefCell;
use std::collections::HashMap;

use util::error::*;
use util::tracing::ResultTraceExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::objects::Duration;

type Callback = Box<dyn FnMut() -> Result<()>>;

thread_local! {
    static TIMER_CALLBACKS: RefCell<HashMap<usize, Callback>> = RefCell::new(HashMap::new());
}

/// Win32-based [Timer]. Needs to be scheduled onto a [EventLoop].
pub struct Timer {
    hwnd: Option<HWND>,
    timer_id: usize,
}

impl Timer {
    /// Create a new [Timer] which calls the callback with the specified due and period.
    pub fn new(period: Duration, cb: Callback) -> Result<Timer> {
        // Store the callback in the thread-local map
        // Set the timer (due is initial delay, period is interval)
        let interval = period.millis();
        let hwnd = None;
        let timer_id = unsafe { SetTimer(hwnd, 0, interval, Some(Some(Self::callback))) };
        TIMER_CALLBACKS.with(|map| {
            map.borrow_mut().insert(timer_id, cb);
        });
        Ok(Timer { hwnd, timer_id })
    }

    unsafe extern "system" fn callback(_hwnd: HWND, _msg: u32, id_timer: usize, _dw_time: u32) {
        TIMER_CALLBACKS.with_borrow_mut(|map| {
            if let Some(cb) = map.get_mut(&id_timer) {
                cb().context("timer callback").error();
            }
        });
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            KillTimer(self.hwnd, self.timer_id)
                .context("kill timer")
                .error();
        }
        TIMER_CALLBACKS.with(|map| {
            map.borrow_mut().remove(&self.timer_id);
        });
    }
}
