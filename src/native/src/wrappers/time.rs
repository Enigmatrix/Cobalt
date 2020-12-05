use crate::error::Win32Err;
use crate::raw::sysinfoapi::GetTickCount64;
use crate::raw::*;
use std::default::default;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Add, Sub};
use std::ptr;
use util::*;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// UTC FILETIME, representing the number of 100-nanosecond intervals since January 1, 1601 (UTC).
pub struct Timestamp(u64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// Duration between Timestamps as the number of 100-nanosecond intervals.
pub struct Duration(i64);

pub struct Timer {
    inner: *mut c_void,
}

static mut BOOT_TIME: MaybeUninit<Timestamp> = MaybeUninit::uninit();

impl Timestamp {
    pub fn now() -> Timestamp {
        let mut ft: minwindef::FILETIME = default();
        unsafe { sysinfoapi::GetSystemTimePreciseAsFileTime(&mut ft) };
        let ticks = unsafe { *(&mut ft as *mut _ as *mut u64) };
        Timestamp(ticks)
    }

    pub fn calculate_boot_time() {
        unsafe {
            BOOT_TIME =
                MaybeUninit::new(Timestamp::now() - Duration::from_millis(GetTickCount64() as i64))
        }
    }

    pub fn from_event_millis(millis: u32) -> Timestamp {
        unsafe { BOOT_TIME.assume_init() + Duration::from_millis(millis as i64) }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut sys: SYSTEMTIME = default();
        unsafe {
            timezoneapi::FileTimeToSystemTime(
                &self.0 as *const _ as *const FILETIME,
                &mut sys as *mut _ as *mut SYSTEMTIME,
            )
        }; // check result?
        write!(
            fmt,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}Z",
            sys.wYear, sys.wMonth, sys.wDay, sys.wHour, sys.wMinute, sys.wSecond, sys.wMilliseconds
        )
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, fmt)
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.0 as u64)
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 - rhs.0 as u64)
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration(self.0 as i64 - rhs.0 as i64)
    }
}

impl Duration {
    pub fn from_millis(millis: i64) -> Duration {
        Duration(millis * 10_000)
    }

    pub fn millis(&self) -> u32 {
        (self.0 / 10_000) as u32
    }
}

impl Timer {
    pub fn new<'a>(
        every_ms: u32,
        cb: Box<dyn 'a + FnMut(Timestamp) -> Result<()>>,
    ) -> Result<Timer, Win32Err> {
        let mut timer = ptr::null_mut();
        let cb = Box::leak(Box::new(cb)) as *mut _ as *mut _;
        win32!(non_zero: threadpoollegacyapiset::CreateTimerQueueTimer(&mut timer, ptr::null_mut(), Some(Timer::callback), cb, 0, every_ms, 0))?;
        Ok(Timer { inner: timer })
    }

    unsafe extern "system" fn callback(arg: *mut c_void, _: u8) {
        let timestamp = Timestamp::now();
        let callback: *mut Box<dyn FnMut(Timestamp) -> Result<()>> = arg.cast();
        (&mut *callback)(timestamp).unwrap_or_exit();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        win32!(non_zero: synchapi::CancelWaitableTimer(self.inner)).unwrap_or_exit();
    }
}
