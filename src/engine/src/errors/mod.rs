use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Win32 (0x{:x}): {}", .0, win32_err(*.0))]
    Win32(i32),
    #[error("HResult (0x{:x})", .0)]
    HResult(i32),
    #[error("NtStatus (0x{:x})", .0)]
    NtStatus(i32),
    #[error("{:?} already closed", .0)]
    WindowAlreadyClosed(crate::os::window::Window),
}

pub use anyhow::*;

fn win32_err(err: i32) -> String {
    std::io::Error::from_raw_os_error(err).to_string()
}

pub fn last_win32_error() -> AppError {
    let err = AppError::Win32(last_win32());
    clear_win32_error();
    err
}

fn clear_win32_error() {
    unsafe { crate::os::api::errhandlingapi::SetLastError(0) }
}

fn last_win32() -> i32 {
    unsafe { crate::os::api::errhandlingapi::GetLastError() as i32 }
}
