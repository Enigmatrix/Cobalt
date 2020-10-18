use crate::os::prelude::*;
use std::fmt;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_ticks(ticks: DWORD) -> Timestamp {
        let millis_diff = ticks as i64 - unsafe { sysinfoapi::GetTickCount64() } as i64;
        Timestamp::now().offset(millis_diff * 10_000)
    }

    pub fn now() -> Timestamp {
        let mut ft: minwindef::FILETIME = default();
        unsafe { sysinfoapi::GetSystemTimePreciseAsFileTime(&mut ft) };
        let ticks = unsafe { *(&mut ft as *mut _ as *mut u64) };
        Timestamp(ticks)
    }

    fn offset(self, amt: i64) -> Timestamp {
        Timestamp((self.0 as i64 + amt) as u64)
    }
}

impl rusqlite::ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Integer(self.0 as i64),
        ))
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
        }; // TODO check result
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
