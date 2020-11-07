use crate::raw::sysinfoapi::GetTickCount64;
use crate::raw::*;
use std::default::default;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Add, Sub};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// UTC FILETIME, representing the number of 100-nanosecond intervals since January 1, 1601 (UTC).
pub struct Timestamp(u64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// Duration between Timestamps as the number of 100-nanosecond intervals.
pub struct Duration(u64);

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
            BOOT_TIME = MaybeUninit::new(Timestamp::now() - Duration::from_millis(GetTickCount64()))
        }
    }

    pub fn from_event_millis(millis: u32) -> Timestamp {
        unsafe { BOOT_TIME.assume_init() + Duration::from_millis(millis as u64) }
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

impl Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.0)
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

impl Duration {
    pub fn from_millis(millis: u64) -> Duration {
        Duration(millis * 10_000)
    }
}
