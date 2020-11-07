use crate::raw::*;
use std::default::default;
use std::fmt;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct Timestamp(u64); // UTC FILETIME

impl Timestamp {
    pub fn now() -> Timestamp {
        let mut ft: minwindef::FILETIME = default();
        unsafe { sysinfoapi::GetSystemTimePreciseAsFileTime(&mut ft) };
        let ticks = unsafe { *(&mut ft as *mut _ as *mut u64) };
        Timestamp(ticks)
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
