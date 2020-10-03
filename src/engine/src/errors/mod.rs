use error_chain::*;

error_chain! {
    errors {
        Win32(res: i32) {
            description("Error occured in Windows API")
            display("Win32 ({}): {}", res, win32_err(*res))
        }
        HResult(res: i32) {}
        NtStatus(res: i32) {}
    }
}

fn win32_err(err: i32) -> String {
    std::io::Error::from_raw_os_error(err).to_string()
}

pub fn last_win32_error() -> Error {
    ErrorKind::Win32(last_win32()).into()
}

pub fn last_win32() -> i32 {
    unsafe { crate::os::api::errhandlingapi::GetLastError() as i32 }
}