use std::mem::MaybeUninit;
use std::{fmt, ops};

use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::SystemInformation::{GetSystemTimePreciseAsFileTime, GetTickCount64};
use windows::Win32::System::Time::FileTimeToSystemTime;

use common::errors::*;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// UTC FILETIME, representing the number of 100-nanosecond intervals since January 1, 1601 (UTC).
pub struct Timestamp(u64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// Duration between Timestamps as the number of 100-nanosecond intervals.
pub struct Duration(i64);

/// The [`Timestamp`] at system boot. Useful as some [`Timestamp`]s given by Win32 calls are based off boot times.
static mut BOOT_TIME: MaybeUninit<Timestamp> = MaybeUninit::uninit();

impl Timestamp {
    /// Get the current [Timestamp]
    pub fn now() -> Timestamp {
        let ft = unsafe { GetSystemTimePreciseAsFileTime() };
        let ticks = unsafe { *(&ft as *const _ as *const u64) };
        Timestamp(ticks)
    }

    /// Find the [`Timestamp`] at system boot by subtracting the current [`Timestamp`] from current time from boot.
    pub(crate) fn setup() -> Result<()> {
        unsafe {
            BOOT_TIME =
                MaybeUninit::new(Timestamp::now() - Duration::from_millis(GetTickCount64() as i64))
        }
        Ok(())
    }

    /// Gets the equivalent [`Timestamp`] given by a Win32 call, based off boot time
    pub fn from_event_millis(millis: u32) -> Timestamp {
        unsafe { BOOT_TIME.assume_init() + Duration::from_millis(millis as i64) }
    }
}

impl From<Timestamp> for u64 {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut sys: SYSTEMTIME = SYSTEMTIME::default();
        unsafe {
            FileTimeToSystemTime(
                &self.0 as *const _ as *const FILETIME,
                &mut sys as *mut _ as *mut SYSTEMTIME,
            )
        };
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

impl ops::Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.0 as u64)
    }
}

impl ops::Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 - rhs.0 as u64)
    }
}

impl ops::Sub<Timestamp> for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration(self.0 as i64 - rhs.0 as i64)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(dur: std::time::Duration) -> Self {
        Duration::from_millis(dur.as_millis() as i64)
    }
}

impl From<Duration> for std::time::Duration {
    fn from(dur: Duration) -> Self {
        std::time::Duration::from_millis(dur.millis() as u64)
    }
}

impl Duration {
    pub const fn from_millis(millis: i64) -> Duration {
        Duration(millis * 10_000)
    }

    pub fn millis(&self) -> u32 {
        (self.0 / 10_000) as u32
    }
}
