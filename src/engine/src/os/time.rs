use crate::os::*;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_ticks(ticks: DWORD) -> Timestamp {
        let mut ft: minwindef::FILETIME = default();
        unsafe { sysinfoapi::GetSystemTimePreciseAsFileTime(&mut ft) };
        let millis_diff = ticks as i64 - unsafe { sysinfoapi::GetTickCount64() } as i64;
        let ticks = unsafe { *(&mut ft as *mut _ as *mut i64) };
        Timestamp((ticks + millis_diff * 10_000) as u64)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
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
