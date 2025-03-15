use std::mem::{self, MaybeUninit};
use std::{fmt, ops};

use util::error::Result;
use util::time::{TimeSystem, ToTicks};
use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::SystemInformation::{GetSystemTimePreciseAsFileTime, GetTickCount64};
use windows::Win32::System::Time::{
    FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime,
    TzSpecificLocalTimeToSystemTime,
};

// https://github.com/dotnet/runtime/blob/333b3d898dbc4372046f9ba74ad912fba62d55ff/src/libraries/System.Private.CoreLib/src/System/DateTime.cs
const MICROSECONDS_PER_MILLISECOND: u64 = 1000;
const TICKS_PER_MICROSECOND: u64 = 10;
const TICKS_PER_MILLISECOND: u64 = TICKS_PER_MICROSECOND * MICROSECONDS_PER_MILLISECOND;

const HOURS_PER_DAY: u64 = 24;
const TICKS_PER_SECOND: u64 = TICKS_PER_MILLISECOND * 1000;
const TICKS_PER_MINUTE: u64 = TICKS_PER_SECOND * 60;
const TICKS_PER_HOUR: u64 = TICKS_PER_MINUTE * 60;
const TICKS_PER_DAY: u64 = TICKS_PER_HOUR * HOURS_PER_DAY;

const DAYS_PER_YEAR: u64 = 365;
const DAYS_PER_4_YEARS: u64 = DAYS_PER_YEAR * 4 + 1;
const DAYS_PER_100_YEARS: u64 = DAYS_PER_4_YEARS * 25 - 1;
const DAYS_PER_400_YEARS: u64 = DAYS_PER_100_YEARS * 4 + 1;
const FILE_TIME_OFFSET: u64 = DAYS_PER_400_YEARS * 4 * TICKS_PER_DAY;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// UTC time representing the number of 100-nanosecond intervals since January 1, 0001 (UTC).
/// NOT the same as FILETIME, which is from 1601 instead.
pub struct Timestamp {
    ticks: u64,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
/// Duration between Timestamps as the number of 100-nanosecond intervals.
pub struct Duration(i64);

/// The [`Timestamp`] at system boot. Useful as some [`Timestamp`]s given by Win32 calls are based off boot times.
static mut BOOT_TIME: MaybeUninit<Timestamp> = MaybeUninit::uninit();

impl Timestamp {
    /// Get the current [Timestamp]
    pub fn now() -> Timestamp {
        let ft = unsafe { GetSystemTimePreciseAsFileTime() };
        let ticks = Self::file_time_to_ticks(ft);
        Timestamp { ticks }
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

    /// Get the system time from the [`Timestamp`]
    fn as_system_time(&self) -> SYSTEMTIME {
        let mut sys: SYSTEMTIME = SYSTEMTIME::default();
        let ft_ticks = self.ticks - FILE_TIME_OFFSET;
        unsafe {
            FileTimeToSystemTime(&ft_ticks as *const _ as *const _, &mut sys)
                .expect("get system time") // should never fail
        };
        sys
    }

    /// Create a [`Timestamp`] from a [`SYSTEMTIME`]
    fn from_system_time(sys: SYSTEMTIME) -> Self {
        let mut ft: FILETIME = FILETIME::default();
        unsafe { SystemTimeToFileTime(&sys as *const _, &mut ft).expect("get file time") }; // should never fail
        let ticks = Self::file_time_to_ticks(ft);
        Timestamp { ticks }
    }

    /// Convert a [`FILETIME`] to ticks, adjust for the difference between 1601 and 0001
    fn file_time_to_ticks(ft: FILETIME) -> u64 {
        let ft_ticks: u64 = unsafe { mem::transmute(ft) };
        ft_ticks + FILE_TIME_OFFSET
    }

    /// Convert [Timestamp] from ticks
    pub fn from_ticks(ticks: i64) -> Self {
        Self {
            ticks: ticks as u64,
        }
    }
}

impl ToTicks for Timestamp {
    fn to_ticks(&self) -> i64 {
        self.ticks as i64
    }
}

fn to_local(sys: &SYSTEMTIME, local_tz: bool) -> SYSTEMTIME {
    if !local_tz {
        return *sys;
    }
    let mut local_sys = SYSTEMTIME::default();
    unsafe {
        SystemTimeToTzSpecificLocalTime(None, sys as *const _, &mut local_sys)
            .expect("convert to local time");
    }
    local_sys
}

fn from_local(local_sys: &SYSTEMTIME, local_tz: bool) -> SYSTEMTIME {
    if !local_tz {
        return *local_sys;
    }
    let mut sys = SYSTEMTIME::default();

    unsafe {
        TzSpecificLocalTimeToSystemTime(None, local_sys as *const _, &mut sys)
            .expect("convert from local time");
    }
    sys
}

impl TimeSystem for Timestamp {
    type Ticks = Self;

    fn now(&self) -> Self {
        *self
    }

    fn day_start(&self, local_tz: bool) -> Self {
        let sys = self.as_system_time();
        let mut local_sys = to_local(&sys, local_tz);
        local_sys.wHour = 0;
        local_sys.wMinute = 0;
        local_sys.wSecond = 0;
        local_sys.wMilliseconds = 0;
        let sys = from_local(&local_sys, local_tz);

        Self::from_system_time(sys)
    }

    fn week_start(&self, local_tz: bool) -> Self {
        let sys = self.as_system_time();
        let mut local_sys = to_local(&sys, local_tz);
        local_sys.wHour = 0;
        local_sys.wMinute = 0;
        local_sys.wSecond = 0;
        local_sys.wMilliseconds = 0;
        let sys = from_local(&local_sys, local_tz);
        let mut time = Self::from_system_time(sys);
        // make monday the start of the week. 0 == sunday
        if local_sys.wDayOfWeek == 0 {
            time.ticks -= 6 * TICKS_PER_DAY;
        } else {
            time.ticks -= (local_sys.wDayOfWeek - 1) as u64 * TICKS_PER_DAY;
        }

        time
    }

    fn month_start(&self, local_tz: bool) -> Self {
        let sys = self.as_system_time();
        let mut local_sys = to_local(&sys, local_tz);
        local_sys.wHour = 0;
        local_sys.wMinute = 0;
        local_sys.wSecond = 0;
        local_sys.wMilliseconds = 0;
        let sys = from_local(&local_sys, local_tz);
        let mut time = Self::from_system_time(sys);
        time.ticks -= (local_sys.wDay - 1) as u64 * TICKS_PER_DAY;
        time
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let sys = self.as_system_time();
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
        Timestamp {
            ticks: self.ticks + rhs.0 as u64,
        }
    }
}

impl ops::Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Timestamp {
            ticks: self.ticks - rhs.0 as u64,
        }
    }
}

impl ops::Sub<Timestamp> for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration(self.ticks as i64 - rhs.ticks as i64)
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
    /// Create a [`Duration`] from milliseconds
    pub const fn from_millis(millis: i64) -> Duration {
        Duration(millis * 10_000)
    }

    /// Create a [`Duration`] from ticks
    pub const fn from_ticks(ticks: i64) -> Duration {
        Duration(ticks)
    }

    /// Get the number of milliseconds in the [`Duration`]
    pub fn millis(&self) -> u32 {
        (self.0 / 10_000) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEB_18_2000_18_05_20: Timestamp = Timestamp {
        ticks: 630864939200000000,
    };
    // 04/03/2000 09:04:02 - Leap Year, Saturday (Sunday (start) is 27 Feb)
    const MAR_04_2000_09_04_02: Timestamp = Timestamp {
        ticks: 630877574420000000,
    };

    #[test]
    fn display_is_correct() {
        assert_eq!("2000-02-18 18:05:20.000Z", FEB_18_2000_18_05_20.to_string());
        assert_eq!("2000-03-04 09:04:02.000Z", MAR_04_2000_09_04_02.to_string());
    }

    #[test]
    fn day_start() {
        assert_eq!(
            "2000-02-18 00:00:00.000Z",
            FEB_18_2000_18_05_20.day_start(false).to_string()
        );
        assert_eq!(
            "2000-03-04 00:00:00.000Z",
            MAR_04_2000_09_04_02.day_start(false).to_string()
        );
    }

    #[test]
    fn week_start() {
        assert_eq!(
            "2000-02-14 00:00:00.000Z",
            FEB_18_2000_18_05_20.week_start(false).to_string()
        );
        assert_eq!(
            "2000-02-28 00:00:00.000Z",
            MAR_04_2000_09_04_02.week_start(false).to_string()
        );
    }

    #[test]
    fn month_start() {
        assert_eq!(
            "2000-02-01 00:00:00.000Z",
            FEB_18_2000_18_05_20.month_start(false).to_string()
        );
        assert_eq!(
            "2000-03-01 00:00:00.000Z",
            MAR_04_2000_09_04_02.month_start(false).to_string()
        );
    }
}
